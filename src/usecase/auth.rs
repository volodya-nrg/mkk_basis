use http::StatusCode;
use std::sync::Arc;
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
    app_errors::AppErr,
};

use super::{UseCaseError, helpers};

// Используется "dyn EmailSender", потому что может приходить как mock-а, так и структура для prod-а.
#[derive(Clone)] // clone из-за axum
pub struct Auth {
    addr: String,
    email_sender: Arc<dyn EmailSender>,
    transactor: Arc<Transactor>,
    users_repo: Arc<DBUsers>,
    pub jwt_service: JWTService, // публичен для экстрактора или middleware. Передаем его по значению, поэтому Arc не нужен.
}

impl Auth {
    pub const fn new(
        addr: String,
        jwt_service: JWTService,
        email_sender: Arc<dyn EmailSender>,
        transactor: Arc<Transactor>,
        users_repo: Arc<DBUsers>,
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
        email: &str,
        password: &str,
        password_confirm: &str,
        agreement: bool,
        privacy_policy: bool,
    ) -> Result<Uuid, UseCaseError> {
        if !HelpersService::is_valid_email(email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailNotCorrect.to_string(),
                internal_err: Some(format!("user send bad email ({})", email)),
            });
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::PasswordIsShort.to_string(),
                internal_err: Default::default(),
            });
        }
        if password != password_confirm {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::PasswordsNotEquals.to_string(),
                internal_err: Default::default(),
            });
        }
        if !agreement {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::NeedAcceptAgreement.to_string(),
                internal_err: Default::default(),
            });
        }
        if !privacy_policy {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::NeedAcceptPrivacyPolicy.to_string(),
                internal_err: Default::default(),
            });
        }

        let code = Uuid::new_v4().simple().to_string();
        let password_hash = helpers::password_hash(password)
            .map_err(|e| UseCaseError::Common(format!("failed to create password hash: {e}")))?;
        let link = format!(
            "{}/register/confirm?email={}&code={}",
            self.addr, email, code
        );
        let email_subject = format!("Confirm email from {}", self.addr);
        let email_message = format!("Confirm email: <a href=\"{}\">{}</a>", link, link);

        Ok(self
            .transactor
            .in_transaction::<_, _, UseCaseError>(async |tx| {
                let user_db = UserDB {
                    email: email.to_string(),
                    password: password_hash.to_string(),
                    email_code: Some(code.clone()),
                    ..Default::default()
                };
                let new_uuid = self.users_repo.create(tx, user_db).await?;

                self.email_sender
                    .send(email, email_subject.as_str(), email_message.as_str())
                    .map_err(|e| UseCaseError::Common(format!("failed to send email: {e}")))?;

                self.email_sender
                    .send(email, email_subject.as_str(), email_message.as_str())
                    .map_err(|e| UseCaseError::Common(format!("failed to send email: {e}")))?;

                Ok(new_uuid)
            })
            .await?)
    }
    pub async fn register_confirm(
        &self,
        email: &str,
        actual_code: &str,
    ) -> Result<(), UseCaseError> {
        if email.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailNotBeEmpty.to_string(),
                internal_err: None,
            });
        }
        if actual_code.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::VerifyCodeNotBeEmpty.to_string(),
                internal_err: None,
            });
        }
        if !HelpersService::is_valid_email(email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailNotCorrect.to_string(),
                internal_err: None,
            });
        }

        let mut db_conn = self.transactor.conn().await?;
        let mut user_db = self.users_repo.by_email(&mut db_conn, email).await?;
        let expected_code = user_db.email_code.ok_or_else(|| UseCaseError::Transport {
            status_code: StatusCode::BAD_REQUEST,
            public_err: AppErr::EmailAlreadyConfirm.to_string(),
            internal_err: None,
        })?;

        if expected_code != *actual_code {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::NotCorrectVerifyEmailCode.to_string(),
                internal_err: None,
            });
        }

        user_db.email_code = None;

        Ok(self.users_repo.update(&mut db_conn, user_db).await?)
    }
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(String, String), UseCaseError> {
        if !HelpersService::is_valid_email(email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailNotCorrect.to_string(),
                internal_err: Some(format!("user send bad email ({})", email)),
            });
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::PasswordIsShort.to_string(),
                internal_err: None,
            });
        }

        let mut db_conn = self.transactor.conn().await?;
        let user_db = self
            .users_repo
            .by_email(&mut db_conn, email)
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
                public_err: AppErr::VerifyYourEmail.to_string(),
                internal_err: None,
            });
        }

        let password_is_eq = helpers::password_verify(password, user_db.password.as_str())
            .map_err(|e| UseCaseError::Common(format!("failed to verify password: {e}")))?;

        if !password_is_eq {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::LoginOrPasswordNotCorrect.to_string(),
                internal_err: None,
            });
        }

        let access_token = self
            .jwt_service
            .generate_access_token(user_db.user_id, user_db.role)?;
        let refresh_token = self.jwt_service.generate_refresh_token(user_db.user_id)?;

        Ok((access_token, refresh_token))
    }
    pub async fn refresh_tokens(&self, token: &str) -> Result<(String, String), UseCaseError> {
        let claims = self
            .jwt_service
            .validate_refresh_token(token)
            .map_err(|e| match e {
                JWTError::ExpiredToken => UseCaseError::Transport {
                    status_code: StatusCode::BAD_REQUEST,
                    public_err: AppErr::TokenExpired.to_string(),
                    internal_err: None,
                },
                _ => UseCaseError::Transport {
                    status_code: StatusCode::BAD_REQUEST,
                    public_err: AppErr::TokenNotValid.to_string(),
                    internal_err: None,
                },
            })?;
        if claims.token_type != TYPE_REFRESH {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::TokenIsNotRefresh.to_string(),
                internal_err: None,
            });
        }

        let mut db_conn = self.transactor.conn().await?;
        let user_db = self.users_repo.one(&mut db_conn, claims.sub).await?;
        let access_token = self
            .jwt_service
            .generate_access_token(user_db.user_id, user_db.role)?;
        let new_refresh_token = self.jwt_service.generate_refresh_token(user_db.user_id)?;

        Ok((access_token, new_refresh_token)) // чтоб пользователь максимально не логинился больше в системе, генерируем новый токен обновления
    }
}
