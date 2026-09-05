use http::StatusCode;
use uuid::Uuid;

use crate::{
    adapter::{
        db::{
            errors::RepositoryError, models::User as UserDB,
            postgres::tables::users::Users as DBUsers,
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
    users_repo: DBUsers,
    pub jwt_service: JWTService, // публичен для экстрактора
    email_sender: ES,
}

impl<ES> Auth<ES>
where
    ES: EmailSender,
{
    pub fn new(
        addr: String,
        users_repo: DBUsers,
        jwt_service: JWTService,
        email_sender: ES,
    ) -> Self {
        Self {
            addr,
            users_repo,
            jwt_service,
            email_sender,
        }
    }
    pub async fn register(
        &self,
        email: String,
        password: String,
        password_confirm: String,
        agreement: bool,
        privacy_policy: bool,
    ) -> Result<Uuid, UseCaseError> {
        if !HelpersService::is_valid_email(&email) {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.to_string(),
                internal_err: Some(format!("user send bad email ({})", email)),
            });
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordIsShort.to_string(),
                internal_err: Default::default(),
            });
        }
        if password != password_confirm {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordsNotEquals.to_string(),
                internal_err: Default::default(),
            });
        }
        if !agreement {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::NeedAcceptAgreement.to_string(),
                internal_err: Default::default(),
            });
        }
        if !privacy_policy {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::NeedAcceptPrivacyPolicy.to_string(),
                internal_err: Default::default(),
            });
        }

        let code = Uuid::new_v4().simple().to_string();
        let password_hash = helpers::password_hash(&password)
            .map_err(|e| UseCaseError::Common(format!("failed to create password hash: {e}")))?;
        let link = format!(
            "{}/register/confirm?email={}&code={}",
            self.addr, email, code
        );
        let email_subject = format!("Confirm email from {}", self.addr);
        let email_message = format!("Confirm email: <a href=\"{}\">{}</a>", link, link);

        // TODO tx
        let result = self
            .users_repo
            .create(UserDB {
                user_id: Default::default(),
                email: email.clone(),
                password: password_hash.to_string(),
                name: None,
                email_code: Some(code.clone()),
                avatar: None,
                role: None,
                created_at: Default::default(),
                updated_at: Default::default(),
            })
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;

        self.email_sender
            .send(email, email_subject.to_string(), email_message.to_string())
            .map_err(|e| UseCaseError::Common(format!("failed to send email: {e}")))?;
        // TODO \tx

        Ok(result)
    }
    pub async fn register_confirm(
        &self,
        email: String,
        actual_code: String,
    ) -> Result<(), UseCaseError> {
        if email.is_empty() {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotBeEmpty.to_string(),
                internal_err: None,
            });
        }
        if actual_code.is_empty() {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::VerifyCodeNotBeEmpty.to_string(),
                internal_err: None,
            });
        }
        if !HelpersService::is_valid_email(&email) {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.to_string(),
                internal_err: None,
            });
        }

        let mut user = match self.users_repo.by_email(email.clone()).await {
            Ok(v) => v,
            Err(e) => {
                if let RepositoryError::NotFoundRow = e {
                    return Err(UseCaseError::ForTransport {
                        status_code: StatusCode::BAD_REQUEST,
                        public_err: ErrMsg::NotFoundUser.to_string(),
                        internal_err: Some(format!("user send other email ({})", email)),
                    });
                }
                return Err(UseCaseError::Common(e.to_string()));
            }
        };
        let expected_code = user.email_code.ok_or_else(|| UseCaseError::ForTransport {
            status_code: StatusCode::BAD_REQUEST,
            public_err: ErrMsg::EmailAlreadyConfirm.to_string(),
            internal_err: None,
        })?;

        if expected_code != actual_code {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::NotCorrectVerifyEmailCode.to_string(),
                internal_err: None,
            });
        }

        user.email_code = None;
        self.users_repo
            .update(user)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed up update: {}", e)))?;

        Ok(())
    }
    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> Result<(String, String), UseCaseError> {
        if !HelpersService::is_valid_email(&email) {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.to_string(),
                internal_err: Some(format!("user send bad email ({})", email)),
            });
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordIsShort.to_string(),
                internal_err: None,
            });
        }

        let user = match self.users_repo.by_email(email.clone()).await {
            Ok(v) => v,
            Err(e) => {
                // если пользователь не найден, то нужно перенаправлять его на страницу регистрации
                if let RepositoryError::NotFoundRow = e {
                    return Err(UseCaseError::UserNotExists);
                }
                return Err(UseCaseError::Common(e.to_string()));
            }
        };

        if user.email_code.is_some() {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::VerifyYourEmail.to_string(),
                internal_err: None,
            });
        }

        let password_is_eq = helpers::password_verify(password.as_str(), user.password.as_str())
            .map_err(|e| UseCaseError::Common(format!("failed to verify password: {e}")))?;

        if !password_is_eq {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::LoginOrPasswordNotCorrect.to_string(),
                internal_err: None,
            });
        }

        let access_token = self
            .jwt_service
            .generate_access_token(user.user_id, user.role)
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        let refresh_token = self
            .jwt_service
            .generate_refresh_token(user.user_id)
            .map_err(|e| UseCaseError::Common(e.to_string()))?;

        Ok((access_token, refresh_token))
    }
    pub async fn logout(&self) -> Result<(), UseCaseError> {
        Ok(())
    }
    pub async fn refresh_tokens(&self, token: String) -> Result<(String, String), UseCaseError> {
        let claims = self
            .jwt_service
            .validate_refresh_token(token)
            .map_err(|e| match e {
                JWTError::ExpiredToken => UseCaseError::ForTransport {
                    status_code: StatusCode::BAD_REQUEST,
                    public_err: ErrMsg::TokenExpired.to_string(),
                    internal_err: None,
                },
                _ => UseCaseError::ForTransport {
                    status_code: StatusCode::BAD_REQUEST,
                    public_err: ErrMsg::TokenNotValid.to_string(),
                    internal_err: None,
                },
            })?;
        if claims.token_type != TYPE_REFRESH {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::TokenIsNotRefresh.to_string(),
                internal_err: None,
            });
        }

        let result = self.users_repo.one(claims.sub).await;
        let user = match result {
            Ok(v) => v,
            Err(e) => {
                if let RepositoryError::NotFoundRow = e {
                    return Err(UseCaseError::ForTransport {
                        status_code: StatusCode::BAD_REQUEST,
                        public_err: ErrMsg::NotFoundUser.to_string(),
                        internal_err: None,
                    });
                }
                return Err(UseCaseError::Common(e.to_string()));
            }
        };

        let access_token = self
            .jwt_service
            .generate_access_token(user.user_id, user.role)
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        let new_refresh_token = self
            .jwt_service
            .generate_refresh_token(user.user_id)
            .map_err(|e| UseCaseError::Common(e.to_string()))?;

        Ok((access_token, new_refresh_token)) // чтоб пользователь максимально не логинился больше в системе, генерируем новый токен обновления
    }
}
