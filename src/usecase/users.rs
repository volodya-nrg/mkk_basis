use super::{UseCaseError, helpers, mapper, models::User};
use crate::adapter::{db::RepositoryError, db::postgres::tables::users::Users as UsersRepo};
use http::StatusCode;
use uuid::Uuid;

#[derive(Clone)] // из-за axum-state
pub struct Users {
    users_repo: UsersRepo,
}

impl Users {
    pub fn new(users_repo: UsersRepo) -> Self {
        Self { users_repo }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<User>, i64), UseCaseError> {
        let (items, total) = self
            .users_repo
            .list(limit, offset)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to get items: {e}")))?;
        Ok((
            items.into_iter().map(mapper::user_db_to_user_uc).collect(),
            total,
        ))
    }
    pub async fn one(&self, user_id: Uuid) -> Result<User, UseCaseError> {
        let user_db = self.users_repo.one(user_id).await.map_err(|e| match e {
            RepositoryError::NotFoundRow => UseCaseError::ForTransport {
                status_code: StatusCode::NOT_FOUND,
                public_err: "item not found".to_string(),
                internal_err: None,
            },
            other => UseCaseError::Common(other.to_string()),
        })?;
        Ok(mapper::user_db_to_user_uc(user_db))
    }
    pub async fn create(&self, mut user: User) -> Result<Uuid, UseCaseError> {
        let password_hash = helpers::password_hash(&user.password)
            .map_err(|e| UseCaseError::Common(format!("failed to create password hash: {e}")))?;
        user.password = password_hash;
        
        let new_uuid = self
            .users_repo
            .create(mapper::user_uc_to_user_db(user))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;
        
        Ok(new_uuid)
    }
    pub async fn update(&self, mut user: User) -> Result<(), UseCaseError> {
        let password_hash = helpers::password_hash(&user.password)
            .map_err(|e| UseCaseError::Common(format!("failed to create password hash: {e}")))?;
        user.password = password_hash;
        
        self.users_repo
            .update(mapper::user_uc_to_user_db(user))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to update: {e}")))?;
        
        Ok(())
    }
    pub async fn delete(&self, user_id: Uuid) -> Result<(), UseCaseError> {
        self.users_repo
            .delete(user_id)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to delete: {e}")))?;
        Ok(())
    }
}
