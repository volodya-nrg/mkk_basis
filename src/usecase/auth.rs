use super::{UseCaseError, helpers, mapper, models::User};
use crate::adapter::{
    db::RepositoryError, db::postgres::tables::users::Users as UsersRepo,
    helpers as helpersService, jwt::Jwt as JWTService,
};
use crate::consts;
use crate::err_msg::ErrMsg;
use http::StatusCode;
use uuid::Uuid;

#[derive(Clone)] // из-за axum-state
pub struct Auth {
    users_repo: UsersRepo,
    pub jwt_service: JWTService, // публичен для экстрактора
}

impl Auth {
    pub fn new(users_repo: UsersRepo, jwt_service: JWTService) -> Self {
        Self {
            users_repo,
            jwt_service,
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
        if !helpersService::is_valid_email(&email) {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.as_str(),
                internal_err: Some(format!("user send bad email ({})", email)),
            });
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordIsShort.as_str(),
                internal_err: Default::default(),
            });
        }
        if password != password_confirm {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordsNotEquals.as_str(),
                internal_err: Default::default(),
            });
        }
        if !agreement {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::NeedAcceptAgreement.as_str(),
                internal_err: Default::default(),
            });
        }
        if !privacy_policy {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::NeedAcceptPrivacyPolicy.as_str(),
                internal_err: Default::default(),
            });
        }

        let password_hash = helpers::password_hash(&password)
            .map_err(|e| UseCaseError::Common(format!("failed to create password hash: {e}")))?;
        let result = self
            .users_repo
            .create(mapper::user_uc_to_user_db(User {
                user_id: Default::default(),
                name: None,
                email,
                password: password_hash.to_string(),
                email_is_confirmed: false,
                avatar: None,
                created_at: Default::default(),
                updated_at: Default::default(),
            }))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;
        
        // TODO отправляем сообщение на е-мэйл
        
        Ok(result)
    }
    pub async fn check_register(
        &self,
    ) -> Result<(), UseCaseError> {
        //1. найти в БД запись о е-мэйле
        Ok(())
    }
    
    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> Result<(String, String), UseCaseError> {
        if !helpersService::is_valid_email(&email) {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.as_str(),
                internal_err: Some(format!("user send bad email ({})", email)),
            });
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordIsShort.as_str(),
                internal_err: None,
            });
        }

        let result = self.users_repo.by_email(email.clone()).await;
        let user = match result {
            Ok(v) => v,
            Err(e) => {
                if let RepositoryError::NotFoundRow = e {
                    return Err(UseCaseError::ForTransport {
                        status_code: StatusCode::BAD_REQUEST,
                        public_err: ErrMsg::NotFoundUser.as_str(),
                        internal_err: Some(format!("user send other email ({})", email)),
                    });
                }
                return Err(UseCaseError::Common(e.to_string()));
            }
        };
        let password_is_eq = helpers::password_verify(password.as_str(), user.password.as_str())
            .map_err(|e| UseCaseError::Common(format!("failed to verify password: {e}")))?;

        if !password_is_eq {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::LoginOrPasswordNotCorrect.as_str(),
                internal_err: None,
            });
        }

        let access_token = self
            .jwt_service
            .generate_access_token(user.user_id, "".to_string())
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
}
