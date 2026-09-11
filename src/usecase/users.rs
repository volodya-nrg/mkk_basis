use http::StatusCode;
use std::fs;
use uuid::Uuid;

use super::{
    UseCaseError, helpers, mapper,
    models::{User, UserCreate, UserUpdate},
};
use crate::adapter::db::{
    models::User as UserDB,
    postgres::tables::users::Users as DBUsers,
    postgres::transactor::{TransactionError, Transactor},
};

#[derive(Clone)] // из-за axum-state
pub struct Users {
    transactor: Transactor,
    users_repo: DBUsers,
}

impl Users {
    pub fn new(transactor: Transactor, users_repo: DBUsers) -> Self {
        Self {
            transactor,
            users_repo,
        }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<User>, i64), UseCaseError> {
        self.transactor
            .in_transaction(async |tx| {
                self.users_repo
                    .list(tx, limit, offset)
                    .await
                    .map_err(|e| UseCaseError::Common(e.to_string()))
                    .map(|list| {
                        (
                            list.0.into_iter().map(mapper::user_db_to_user_uc).collect(),
                            list.1,
                        )
                    })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Database(sqlx_err) => UseCaseError::Common(sqlx_err.to_string()),
                TransactionError::Operation(use_case_err) => use_case_err,
            })
    }
    pub async fn one(&self, item_id: Uuid) -> Result<User, UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;

        Ok(mapper::user_db_to_user_uc(
            self.users_repo.one(&mut db_conn, item_id).await?,
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

        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;

        Ok(self
            .users_repo
            .create(
                &mut db_conn,
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
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        let user_db = self.users_repo.one(&mut db_conn, user.user_id).await?;
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
        self.transactor
            .in_transaction(async |tx| {
                self.users_repo
                    .update(tx, user_db_copy)
                    .await
                    .map_err(|e| UseCaseError::Common(e.to_string()))
                    .and_then(|_| {
                        if let Some(v) = user_db.avatar.clone()
                            && let Err(e) = fs::remove_file(v.clone())
                        {
                            return Err(UseCaseError::Common(format!(
                                "failed to remove file ({v}): {e}",
                            )));
                        }
                        Ok(())
                    })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Database(sqlx_err) => UseCaseError::Common(sqlx_err.to_string()),
                TransactionError::Operation(use_case_err) => use_case_err,
            })
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        let user = self.users_repo.one(&mut db_conn, item_id).await?;

        self.transactor
            .in_transaction(async |tx| {
                self.users_repo
                    .delete(tx, item_id)
                    .await
                    .map_err(|e| UseCaseError::Common(e.to_string()))
                    .and_then(|_| {
                        if let Some(v) = user.avatar
                            && let Err(e) = fs::remove_file(v.clone())
                        {
                            return Err(UseCaseError::Common(format!(
                                "failed to remove file ({v}): {e}",
                            )));
                        }
                        Ok(())
                    })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Database(sqlx_err) => UseCaseError::Common(sqlx_err.to_string()),
                TransactionError::Operation(use_case_err) => use_case_err,
            })
    }
    fn create_password_hash(&self, pass: String) -> Result<String, UseCaseError> {
        helpers::password_hash(&pass)
            .map_err(|e| UseCaseError::Common(format!("failed to create password-hash: {e}")))
    }
}
