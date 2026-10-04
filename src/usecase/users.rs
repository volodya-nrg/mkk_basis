use http::StatusCode;
use std::fs;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapter::db::{models::User as UserDB, storage::Storage};

use super::{
    UseCaseError, helpers, mapper,
    models::{User, UserCreate, UserUpdate},
};

#[derive(Clone)] // clone из-за axum
pub struct Users {
    storage: Arc<dyn Storage>,
}

impl Users {
    pub const fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<User>, i64), UseCaseError> {
        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;
        let list = self.storage.users().list(&mut conn, limit, offset).await?;

        tx.commit().await?;
        Ok((
            list.0.into_iter().map(mapper::user_db_to_user_uc).collect(),
            list.1,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<User, UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(mapper::user_db_to_user_uc(
            self.storage
                .users()
                .one(&mut conn.as_mut(), item_id)
                .await?,
        ))
    }
    pub async fn create(&self, mut user: UserCreate) -> Result<Uuid, UseCaseError> {
        if user.email.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: "email is require".to_string(),
                internal_err: None,
            });
        }
        if user.password.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: "password is require".to_string(),
                internal_err: None,
            });
        }

        user.password = self.create_password_hash(user.password)?;
        let mut conn = self.storage.get_conn().await?;

        Ok(self
            .storage
            .users()
            .create(
                &mut conn.as_mut(),
                UserDB {
                    user_id: Default::default(),
                    email: user.email,
                    password: user.password,
                    name: user.name,
                    email_code: user.email_code,
                    avatar: user.avatar,
                    role: user.role,
                    created_at: Default::default(),
                    updated_at: Default::default(),
                },
            )
            .await?)
    }
    pub async fn update(&self, user: UserUpdate) -> Result<(), UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        let user_db = self
            .storage
            .users()
            .one(&mut conn.as_mut(), user.user_id)
            .await?;
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
        if user_db == user_db_copy {
            return Ok(());
        }

        // если файл удалился нормально, то транзакция завершена
        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;

        self.storage.users().update(&mut conn, user_db_copy).await?;

        if user.is_remove_avatar
            && let Some(avatar_filepath) = user_db.avatar.clone()
            && let Err(e) = fs::remove_file(avatar_filepath.clone())
        {
            return Err(UseCaseError::Common(format!(
                "failed to remove file ({avatar_filepath}): {e}",
            )));
        }

        tx.commit().await?;
        Ok(())
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        let user = self
            .storage
            .users()
            .one(&mut conn.as_mut(), item_id)
            .await?;
        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;

        self.storage.users().delete(&mut conn, item_id).await?;

        if let Some(v) = user.avatar
            && let Err(e) = fs::remove_file(v.clone())
        {
            return Err(UseCaseError::Common(format!(
                "failed to remove file ({v}): {e}",
            )));
        }

        tx.commit().await?;
        Ok(())
    }
    fn create_password_hash(&self, pass: String) -> Result<String, UseCaseError> {
        helpers::password_hash(&pass)
            .map_err(|e| UseCaseError::Common(format!("failed to create password-hash: {e}")))
    }
}
