use super::{UseCaseError, mapper, models::*};
use crate::adapter::db::RepositoryError;
use crate::adapter::db::postgres::tables::users::Users as UsersRepo;
use crate::adapter::helpers;
use crate::adapter::jwt::Jwt as JWTService;
use crate::consts;
use crate::err_msg::ErrMsg;
use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
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
    ) -> Result<Uuid, UseCaseError> {
        if !helpers::is_valid_email(&email) {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.as_str(),
                internal_err: format!("user send bad email ({})", email),
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

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| UseCaseError::Common(format!("failed to hash password: {e}")))?;
        let result = self
            .users_repo
            .create(mapper::user_uc_to_user_db(User {
                user_id: Default::default(),
                name: None,
                email,
                password: password_hash.to_string(),
                email_is_confirmed: false,
                created_at: Default::default(),
                updated_at: Default::default(),
            }))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;

        // TODO тут надо отправить сообщение на е-мэйл с ссылкой для подтверждения пароля

        Ok(result)
    }
    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> Result<(String, String), UseCaseError> {
        if !helpers::is_valid_email(&email) {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::EmailNotCorrect.as_str(),
                internal_err: format!("user send bad email ({})", email),
            });
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::PasswordIsShort.as_str(),
                internal_err: String::new(),
            });
        }

        let result = self.users_repo.get_by_email(email.clone()).await;
        let user = match result {
            Ok(v) => v,
            Err(e) => {
                if let RepositoryError::NotFoundRow = e {
                    return Err(UseCaseError::ForTransport {
                        status_code: StatusCode::BAD_REQUEST,
                        public_err: ErrMsg::NotFoundUser.as_str(),
                        internal_err: format!("user send other email ({})", email),
                    });
                }
                return Err(UseCaseError::Common(e.to_string()));
            }
        };

        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| UseCaseError::Common(format!("failed to create new parsed hash: {e}")))?;
        let is_eq = Argon2::default().verify_password(password.as_ref(), &parsed_hash);

        if !is_eq.is_ok() {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: ErrMsg::LoginOrPasswordNotCorrect.as_str(),
                internal_err: String::new(),
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
