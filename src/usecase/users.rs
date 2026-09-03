use http::StatusCode;
use std::fs;
use uuid::Uuid;

use crate::adapter::db::{models::User as UserDB, postgres::tables::users::Users as DBUsers};

use super::{
    UseCaseError, helpers, mapper,
    models::{User, UserCreate, UserUpdate},
};

#[derive(Clone)] // из-за axum-state
pub struct Users {
    users_repo: DBUsers,
}

impl Users {
    pub fn new(users_repo: DBUsers) -> Self {
        Self { users_repo }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<User>, i64), UseCaseError> {
        self.users_repo
            .list(limit, offset)
            .await
            .map_err(|e| e.into())
            .map(|(items, total)| {
                (
                    items.into_iter().map(mapper::user_db_to_user_uc).collect(),
                    total,
                )
            })
    }
    pub async fn one(&self, item_id: Uuid) -> Result<User, UseCaseError> {
        Ok(mapper::user_db_to_user_uc(
            self.users_repo.one(item_id).await?, // тут срабатывает авто конвертация
        ))
    }
    pub async fn create(&self, mut user: UserCreate) -> Result<Uuid, UseCaseError> {
        if user.email.is_empty() {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: "email is require".to_string(),
                internal_err: None,
            });
        }
        if user.password.is_empty() {
            return Err(UseCaseError::ForTransport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: "password is require".to_string(),
                internal_err: None,
            });
        }

        user.password = self.create_password_hash(user.password)?;

        self.users_repo
            .create(UserDB {
                user_id: Default::default(),
                email: user.email,
                password: user.password,
                name: user.name,
                email_code: user.email_code,
                avatar: user.avatar,
                role: user.role,
                created_at: Default::default(),
                updated_at: Default::default(),
            })
            .await
            .map_err(|e| e.into())
    }
    pub async fn update(&self, user: UserUpdate) -> Result<(), UseCaseError> {
        let user_db = self.users_repo.one(user.user_id).await?;
        let mut user_db_copy = user_db.clone();

        if let Some(v) = user.email {
            user_db_copy.email = v;
        }
        if let Some(v) = user.password {
            user_db_copy.password = self.create_password_hash(v)?;
        }
        if let Some(v) = user.name {
            user_db_copy.name = Some(v);
        }
        if let Some(v) = user.email_code {
            user_db_copy.email_code = Some(v);
        }
        if let Some(v) = user.role {
            user_db_copy.role = Some(v);
        }
        if user.is_remove_avatar && user_db.avatar.is_some() {
            user_db_copy.avatar = None;
        }
        if let Some(v) = user.avatar {
            user_db_copy.avatar = Some(v);
        }
        if user_db != user_db_copy {
            // если файл удалился нормально, то транзакция завершена
            // TODO tx
            self.users_repo.update(user_db_copy).await?;
            if let Some(v) = user_db.avatar.clone()
                && let Err(e) = fs::remove_file(v.clone())
            {
                log::error!("failed to remove file ({}): {}", v, e)
            }
            // TODO \tx
        }

        Ok(())
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        let user = self.users_repo.one(item_id).await?;

        // TODO tx
        self.users_repo.delete(item_id).await?;
        if let Some(v) = user.avatar
            && let Err(e) = fs::remove_file(v.clone())
        {
            log::error!("failed to remove file ({}): {}", v, e);
        }
        // TODO \tx

        Ok(())
    }
    fn create_password_hash(&self, pass: String) -> Result<String, UseCaseError> {
        helpers::password_hash(&pass)
            .map_err(|e| UseCaseError::Common(format!("failed to create password-hash: {e}")))
    }
}
