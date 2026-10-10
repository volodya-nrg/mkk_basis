use anyhow::Context;
use http::StatusCode;
use std::fs;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapter::db::{models::User as UserDB, storage::Storage};

use super::{
    errors::UseCaseError,
    helpers, mapper,
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
    pub async fn list(&self, limit: i32, offset: i32) -> anyhow::Result<(Vec<User>, i64)> {
        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;
        let list = self
            .storage
            .users()
            .list(&mut conn, limit, offset)
            .await
            .context("failed to get list users")?;

        tx.commit().await.context("failed to tx-commit")?;
        Ok((
            list.0.into_iter().map(mapper::user_db_to_user_uc).collect(),
            list.1,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> anyhow::Result<User> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        Ok(mapper::user_db_to_user_uc(
            self.storage
                .users()
                .one(&mut conn.as_mut(), item_id)
                .await
                .map_err(UseCaseError::from)
                .context("failed to get user")?,
        ))
    }
    // в Result лучше тип-ошибки не ставить, а делать через thiserror и into(). Так охватываем несколько вариантов отдачи.
    pub async fn create(&self, mut user: UserCreate) -> anyhow::Result<Uuid> {
        if user.email.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: "email is require".to_string(),
                internal_err: None,
            }
            .into());
        }
        if user.password.is_empty() {
            return Err(UseCaseError::Transport {
                status_code: StatusCode::BAD_REQUEST,
                public_err: "password is require".to_string(),
                internal_err: None,
            }
            .into());
        }

        user.password = self.create_password_hash(user.password)?; // тут можно без context-а
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        self.storage
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
            .await
            .context("failed to create user")
    }
    pub async fn update(&self, user: UserUpdate) -> anyhow::Result<()> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;
        let user_db = self
            .storage
            .users()
            .one(&mut conn.as_mut(), user.user_id)
            .await
            .map_err(UseCaseError::from)
            .context("failed to get user")?;
        let mut user_db_copy = user_db.clone();

        if let Some(v) = user.email {
            user_db_copy.email = v;
        }
        if let Some(v) = user.password {
            user_db_copy.password = self.create_password_hash(v)?; // тут можно без context-а
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
        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;

        self.storage
            .users()
            .update(&mut conn, user_db_copy)
            .await
            .context("failed to update user")?;

        if user.is_remove_avatar
            && let Some(avatar_filepath) = user_db.avatar.clone()
            && let Err(e) = fs::remove_file(avatar_filepath.clone())
        {
            return Err(anyhow::Error::from(e)
                .context(format!("failed to remove file ({avatar_filepath})")));
        }

        tx.commit().await.context("failed to tx-commit")
    }
    pub async fn delete(&self, item_id: Uuid) -> anyhow::Result<()> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;
        let user = self
            .storage
            .users()
            .one(&mut conn.as_mut(), item_id)
            .await
            .map_err(UseCaseError::from)
            .context("failed to get user")?;
        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;

        self.storage
            .users()
            .delete(&mut conn, item_id)
            .await
            .context("failed to delete user")?;

        if let Some(v) = user.avatar
            && let Err(e) = fs::remove_file(v.clone())
        {
            return Err(anyhow::Error::from(e).context(format!("failed to remove file ({v})")));
        }

        tx.commit().await.context("failed to tx-commit")
    }
    fn create_password_hash(&self, pass: String) -> anyhow::Result<String> {
        helpers::password_hash(&pass).context("failed to create password-hash")
    }
}
