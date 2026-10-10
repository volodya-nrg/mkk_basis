use anyhow::{Context, anyhow};
use http::StatusCode;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    adapter::{
        db::{models::User as UserDB, storage::Storage},
        email::{ConfirmationCodeStorer, EmailSender},
        helpers as HelpersService,
        jwt::{JWTError, Jwt as JWTService, TYPE_REFRESH},
    },
    app_errors::AppErr,
    consts,
};

use super::{errors::UseCaseError, helpers};

// Используется "dyn EmailSender", потому что может приходить как mock-а, так и структура для prod-а.
#[derive(Clone)] // clone из-за axum
pub struct Auth {
    addr: String,
    email_sender: Arc<dyn EmailSender>,
    codes: Arc<dyn ConfirmationCodeStorer>,
    storage: Arc<dyn Storage>,
    pub jwt_service: JWTService, // публичен для экстрактора или middleware. Передаем его по значению, поэтому Arc не нужен.
}

impl Auth {
    pub const fn new(
        addr: String,
        jwt_service: JWTService,
        email_sender: Arc<dyn EmailSender>,
        codes: Arc<dyn ConfirmationCodeStorer>,
        storage: Arc<dyn Storage>,
    ) -> Self {
        Self {
            addr,
            jwt_service,
            email_sender,
            codes,
            storage,
        }
    }
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        password_confirm: &str,
        agreement: bool,
        privacy_policy: bool,
    ) -> anyhow::Result<Uuid> {
        if !HelpersService::is_valid_email(email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailNotCorrect.to_string(),
                internal_err: Some(format!("user send bad email ({})", email)),
            }
            .into());
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::PasswordIsShort.to_string(),
                internal_err: None,
            }
            .into());
        }
        if password != password_confirm {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::PasswordsNotEquals.to_string(),
                internal_err: None,
            }
            .into());
        }
        if !agreement {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::NeedAcceptAgreement.to_string(),
                internal_err: None,
            }
            .into());
        }
        if !privacy_policy {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::NeedAcceptPrivacyPolicy.to_string(),
                internal_err: None,
            }
            .into());
        }

        let code = Uuid::new_v4().simple().to_string();
        let password_hash = helpers::password_hash(password)
            .map_err(|e| anyhow!("failed to create password hash: {e}"))?;
        let link = format!(
            "{}/register/confirm?email={}&code={}",
            self.addr, email, code
        );
        let email_subject = format!("Confirm email from {}", self.addr);
        let email_message = format!("Confirm email: <a href=\"{}\">{}</a>", link, link);
        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;
        let user_db = UserDB {
            email: email.to_string(),
            password: password_hash.to_string(),
            email_code: Some(code.clone()),
            ..Default::default()
        };
        let new_uuid = self
            .storage
            .users()
            .create(&mut conn, user_db)
            .await
            .context("failed to create user")?;

        self.email_sender
            .send(email, email_subject.as_str(), email_message.as_str())
            .context("failed to send email")?;
        self.codes.store(email, code.as_str()); // сохраним для теста

        tx.commit().await.context("failed to tx-commit")?;
        Ok(new_uuid)
    }
    pub async fn register_confirm(&self, email: &str, actual_code: &str) -> anyhow::Result<()> {
        if email.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailNotBeEmpty.to_string(),
                internal_err: None,
            }
            .into());
        }
        if actual_code.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::VerifyCodeNotBeEmpty.to_string(),
                internal_err: None,
            }
            .into());
        }
        if !HelpersService::is_valid_email(email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailNotCorrect.to_string(),
                internal_err: None,
            }
            .into());
        }

        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;
        let mut user_db = self
            .storage
            .users()
            .by_email(&mut conn.as_mut(), email)
            .await
            .map_err(UseCaseError::from)
            .context("failed to get user by email")?;
        let expected_code = user_db
            .email_code
            .ok_or_else(|| UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailAlreadyConfirm.to_string(),
                internal_err: None,
            })
            .context("email is none")?;

        if expected_code != *actual_code {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::NotCorrectVerifyEmailCode.to_string(),
                internal_err: None,
            }
            .into());
        }

        user_db.email_code = None;
        self.storage
            .users()
            .update(&mut conn.as_mut(), user_db)
            .await
            .context("failed to update user")
    }
    pub async fn login(&self, email: &str, password: &str) -> anyhow::Result<(String, String)> {
        if !HelpersService::is_valid_email(email) {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::EmailNotCorrect.to_string(),
                internal_err: Some(format!("user send bad email ({})", email)),
            }
            .into());
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::PasswordIsShort.to_string(),
                internal_err: None,
            }
            .into());
        }

        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;
        let user_db = self
            .storage
            .users()
            .by_email(&mut conn.as_mut(), email)
            .await
            .map_err(UseCaseError::from)
            .context("failed to get user by email")?;

        if user_db.email_code.is_some() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::VerifyYourEmail.to_string(),
                internal_err: None,
            }
            .into());
        }

        let password_is_eq = helpers::password_verify(password, user_db.password.as_str())
            .context("failed to verify password")?;

        if !password_is_eq {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::LoginOrPasswordNotCorrect.to_string(),
                internal_err: None,
            }
            .into());
        }

        let access_token = self
            .jwt_service
            .generate_access_token(user_db.user_id, user_db.role)
            .context("failed to gen access-token")?;
        let refresh_token = self
            .jwt_service
            .generate_refresh_token(user_db.user_id)
            .context("failed to gen refresh-token")?;

        Ok((access_token, refresh_token))
    }
    pub async fn refresh_tokens(&self, token: &str) -> anyhow::Result<(String, String)> {
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
            })
            .context("failed to validate refresh-token")?;

        if claims.token_type != TYPE_REFRESH {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: AppErr::TokenIsNotRefresh.to_string(),
                internal_err: None,
            }
            .into());
        }

        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;
        let user_db = self
            .storage
            .users()
            .one(&mut conn.as_mut(), claims.sub)
            .await
            .map_err(UseCaseError::from)
            .context("failed to get user")?;
        let access_token = self
            .jwt_service
            .generate_access_token(user_db.user_id, user_db.role)
            .context("failed to gen access-token")?;
        let new_refresh_token = self
            .jwt_service
            .generate_refresh_token(user_db.user_id)
            .context("failed to gen refresh-token")?;

        Ok((access_token, new_refresh_token)) // чтоб пользователь максимально не логинился больше в системе, генерируем новый токен обновления
    }
}
