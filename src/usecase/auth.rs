use super::{mapper, models::*};
use crate::adapter::db::errors::Error;
use crate::adapter::db::postgres::tables::users::Users as UsersRepo;
use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use uuid::Uuid;

#[derive(Clone)] // из-за axum-state
pub struct Auth {
    users_repo: UsersRepo,
}

impl Auth {
    pub fn new(users_repo: UsersRepo) -> Self {
        Self { users_repo }
    }
    pub async fn register(&self, email: String, password: String) -> Result<Uuid, String> {
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
                email_is_confirmed: true, // TODO явно поставить пока true
                created_at: Default::default(),
                updated_at: Default::default(),
            }))
            .await
            .map_err(|e| format!("failed to create: {e}"))?;
        Ok(result)
    }
    pub async fn login(&self, email: String, password: String) -> Result<(String, String), String> {
        let result = self.users_repo.get_by_email(email.clone()).await;
        let user = match result {
            Ok(v) => v,
            Err(e) => {
                return match e {
                    Error::NotFound => Err(format!("not found user({})", email)),
                    Error::Any(e) => Err(format!("failed to get user: {e}")),
                };
            }
        };
        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| format!("failed to create new parsed hash: {e}"))?;
        let is_eq = Argon2::default().verify_password(password.as_ref(), &parsed_hash);

        if !is_eq.is_ok() {
            return Err("not correct password".to_string());
        }

        let access_token = String::new();
        let refresh_token = String::new();

        Ok((access_token, refresh_token))
    }
    pub async fn logout(&self) -> Result<(), String> {
        Ok(())
    }
}
