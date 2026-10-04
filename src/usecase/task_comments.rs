use std::sync::Arc;
use uuid::Uuid;

use crate::adapter::db::storage::Storage;

use super::{UseCaseError, mapper, models::TaskComment};

#[derive(Clone)] // clone из-за axum
pub struct TaskComments {
    storage: Arc<dyn Storage>,
}

impl TaskComments {
    pub const fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }
    pub async fn list(
        &self,
        task_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<TaskComment>, i64), UseCaseError> {
        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;
        let list = self
            .storage
            .task_comments()
            .list(&mut conn, task_id, limit, offset)
            .await?;
        tx.commit().await?;

        Ok((
            list.0
                .into_iter() // по значениям
                .map(mapper::task_comment_db_to_task_comment_uc)
                .collect(),
            list.1,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<TaskComment, UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(mapper::task_comment_db_to_task_comment_uc(
            self.storage
                .task_comments()
                .one(&mut conn.as_mut(), item_id)
                .await?,
        ))
    }
    pub async fn create(&self, task_comment: TaskComment) -> Result<Uuid, UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(self
            .storage
            .task_comments()
            .create(
                &mut conn.as_mut(),
                mapper::task_comment_uc_to_task_comment_db(task_comment),
            )
            .await?)
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(self
            .storage
            .task_comments()
            .delete(&mut conn.as_mut(), item_id)
            .await?)
    }
}
