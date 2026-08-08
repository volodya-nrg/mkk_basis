use super::{mapper, models::*};
use crate::adapter::db::errors::Error as DBError;
use crate::adapter::db::postgres::tables::users::Users as UsersRepo;
use crate::adapter::helpers;
use crate::consts;
use crate::custom_error::CustomError;
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
}

impl Auth {
    pub fn new(users_repo: UsersRepo) -> Self {
        Self { users_repo }
    }
    pub async fn register(
        &self,
        email: String,
        password: String,
        password_confirm: String,
    ) -> Result<Uuid, String> {
        if !helpers::is_valid_email(&email) {
            return Err(
                CustomError::new(StatusCode::BAD_REQUEST, ErrMsg::EmailNotCorrect.as_str()).into(),
            );
        }
        if password.chars().count() < consts::MIN_PASSWORD_LEN {
            return Err(
                CustomError::new(StatusCode::BAD_REQUEST, ErrMsg::PasswordIsShort.as_str()).into(),
            );
        }
        if password != password_confirm {
            return Err(CustomError::new(
                StatusCode::BAD_REQUEST,
                ErrMsg::PasswordsNotEquals.as_str(),
            )
            .into());
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("failed to hash password: {e}"))?;
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
            .map_err(|e| format!("failed to create: {e}"))?;

        // TODO тут надо отправить сообщение на е-мэйл с ссылкой для подтверждения пароля

        Ok(result)
    }
    pub async fn login(&self, email: String, password: String) -> Result<(String, String), String> {
        let result = self.users_repo.get_by_email(email.clone()).await;
        let user = match result {
            Ok(v) => v,
            Err(e) => {
                return match e {
                    DBError::NotFound => Err(format!("not found user ({})", email)),
                    DBError::Any(e) => Err(format!("failed to get user: {e}")),
                };
            }
        };

        // TODO это дело пока опустим, после надо будет доделать
        // if !user.email_is_confirmed {
        //     return Err("email is not confirmed".to_string());
        // }

        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| format!("failed to create new parsed hash: {e}"))?;
        let is_eq = Argon2::default().verify_password(password.as_ref(), &parsed_hash);

        if !is_eq.is_ok() {
            return Err("not correct password".to_string());
        }

        let access_token = String::from("access_token");
        let refresh_token = String::from("refresh_token");

        Ok((access_token, refresh_token))
    }
    pub async fn logout(&self) -> Result<(), String> {
        Ok(())
    }
}
