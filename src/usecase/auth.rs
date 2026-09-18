use http::StatusCode;
use uuid::Uuid;

use crate::{
    adapter::{
        db::{
            errors::RepositoryError,
            models::User as UserDB,
            postgres::{tables::users::Users as DBUsers, transactor::Transactor},
        },
        email::EmailSender,
        helpers as HelpersService,
        jwt::{JWTError, Jwt as JWTService, TYPE_REFRESH},
    },
    consts,
    err_msg::ErrMsg,
};

use super::{UseCaseError, helpers};

#[derive(Clone)] // из-за axum-state
pub struct Auth<ES> {
    addr: String,
    email_sender: ES,
    transactor: Transactor,
    users_repo: DBUsers,

    pub jwt_service: JWTService, // публичен для экстрактора или middleware
}

impl<ES> Auth<ES>
where
    ES: EmailSender,
{
    pub const fn new(
        addr: String,
        jwt_service: JWTService,
        email_sender: ES,
        transactor: Transactor,
        users_repo: DBUsers,
    ) -> Self {
        Self {
            addr,
            jwt_service,
            email_sender,
            transactor,
            users_repo,
        }
    }
    pub async fn register(
        &self,
        ref_email: &str,
        ref_password: &str,
        ref_password_confirm: &str,
        agreement: bool,
        privacy_policy: bool,
    ) -> Result<Uuid, UseCaseError> {
        if !HelpersService::is_valid_email(ref_email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.to_string(),
                internal_err: Some(format!("user send bad email ({})", ref_email)),
            });
        }
        if ref_password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordIsShort.to_string(),
                internal_err: Default::default(),
            });
        }
        if ref_password != ref_password_confirm {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordsNotEquals.to_string(),
                internal_err: Default::default(),
            });
        }
        if !agreement {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::NeedAcceptAgreement.to_string(),
                internal_err: Default::default(),
            });
        }
        if !privacy_policy {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::NeedAcceptPrivacyPolicy.to_string(),
                internal_err: Default::default(),
            });
        }

        let code = Uuid::new_v4().simple().to_string();
        let password_hash = helpers::password_hash(ref_password)
            .map_err(|e| UseCaseError::Common(format!("failed to create password hash: {e}")))?;
        let link = format!(
            "{}/register/confirm?email={}&code={}",
            self.addr, ref_email, code
        );
        let email_subject = format!("Confirm email from {}", self.addr);
        let email_message = format!("Confirm email: <a href=\"{}\">{}</a>", link, link);

        Ok(self
            .transactor
            .in_transaction::<_, _, UseCaseError>(async |tx| {
                let user_db = UserDB {
                    // так линтер советует
                    email: ref_email.to_string(),
                    password: password_hash.to_string(),
                    email_code: Some(code),
                    ..Default::default()
                };
                let new_uuid = self.users_repo.create(tx, user_db).await?;

                self.email_sender
                    .send(ref_email, email_subject.as_str(), email_message.as_str())
                    .map_err(|e| UseCaseError::Common(format!("failed to send email: {e}")))?;

                Ok(new_uuid)
            })
            .await?)
    }
    pub async fn register_confirm(
        &self,
        ref_email: &str,
        ref_actual_code: &str,
    ) -> Result<(), UseCaseError> {
        if ref_email.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotBeEmpty.to_string(),
                internal_err: None,
            });
        }
        if ref_actual_code.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::VerifyCodeNotBeEmpty.to_string(),
                internal_err: None,
            });
        }
        if !HelpersService::is_valid_email(ref_email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.to_string(),
                internal_err: None,
            });
        }

        let mut db_conn = self.transactor.conn().await?;
        let mut user_db = self.users_repo.by_email(&mut db_conn, ref_email).await?;
        let expected_code = user_db.email_code.ok_or_else(|| UseCaseError::Transport {
            status_code: StatusCode::BAD_REQUEST,
            public_err: ErrMsg::EmailAlreadyConfirm.to_string(),
            internal_err: None,
        })?;

        if expected_code != *ref_actual_code {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::NotCorrectVerifyEmailCode.to_string(),
                internal_err: None,
            });
        }

        user_db.email_code = None;

        Ok(self.users_repo.update(&mut db_conn, user_db).await?)
    }
    pub async fn login(
        &self,
        ref_email: &str,
        ref_password: &str,
    ) -> Result<(String, String), UseCaseError> {
        if !HelpersService::is_valid_email(ref_email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.to_string(),
                internal_err: Some(format!("user send bad email ({})", ref_email)),
            });
        }
        if ref_password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordIsShort.to_string(),
                internal_err: None,
            });
        }

        let mut db_conn = self.transactor.conn().await?;
        let user_db = self
            .users_repo
            .by_email(&mut db_conn, ref_email)
            .await
            .map_err(|e| {
                // ! если пользователь не найден, то нужно перенаправлять его на страницу регистрации - тут исключение
                if matches!(e, RepositoryError::NotFoundRow) {
                    return UseCaseError::UserNotExists;
                }
                UseCaseError::Common(e.to_string())
            })?;

        if user_db.email_code.is_some() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::VerifyYourEmail.to_string(),
                internal_err: None,
            });
        }

        let password_is_eq = helpers::password_verify(ref_password, user_db.password.as_str())
            .map_err(|e| UseCaseError::Common(format!("failed to verify password: {e}")))?;

        if !password_is_eq {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::LoginOrPasswordNotCorrect.to_string(),
                internal_err: None,
            });
        }

        let access_token = self
            .jwt_service
            .generate_access_token(&user_db.user_id, &user_db.role)?;
        let refresh_token = self.jwt_service.generate_refresh_token(&user_db.user_id)?;

        Ok((access_token, refresh_token))
    }
    pub async fn refresh_tokens(&self, ref_token: &str) -> Result<(String, String), UseCaseError> {
        let claims = self
            .jwt_service
            .validate_refresh_token(ref_token)
            .map_err(|e| match e {
                JWTError::ExpiredToken => UseCaseError::Transport {
                    status_code: StatusCode::BAD_REQUEST,
                    public_err: ErrMsg::TokenExpired.to_string(),
                    internal_err: None,
                },
                _ => UseCaseError::Transport {
                    status_code: StatusCode::BAD_REQUEST,
                    public_err: ErrMsg::TokenNotValid.to_string(),
                    internal_err: None,
                },
            })?;
        if claims.token_type != TYPE_REFRESH {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::TokenIsNotRefresh.to_string(),
                internal_err: None,
            });
        }

        let mut db_conn = self.transactor.conn().await?;
        let user_db = self.users_repo.one(&mut db_conn, &claims.sub).await?;
        let access_token = self
            .jwt_service
            .generate_access_token(&user_db.user_id, &user_db.role)?;
        let new_refresh_token = self.jwt_service.generate_refresh_token(&user_db.user_id)?;

        Ok((access_token, new_refresh_token)) // чтоб пользователь максимально не логинился больше в системе, генерируем новый токен обновления
    }
}
