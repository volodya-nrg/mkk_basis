use anyhow::Context;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapter::db::storage::Storage;
use crate::usecase::errors::UseCaseError;
use super::{mapper, models::TaskComment};

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
    ) -> anyhow::Result<(Vec<TaskComment>, i64)> {
        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;
        let list = self
            .storage
            .task_comments()
            .list(&mut conn, task_id, limit, offset)
            .await
            .context("failed to get list task-comments")?;

        tx.commit().await.context("failed to tx-commit")?;
        Ok((
            list.0
                .into_iter() // по значениям
                .map(mapper::task_comment_db_to_task_comment_uc)
                .collect(),
            list.1,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> anyhow::Result<TaskComment> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        Ok(mapper::task_comment_db_to_task_comment_uc(
            self.storage
                .task_comments()
                .one(&mut conn.as_mut(), item_id)
                .await
                .map_err(UseCaseError::from)
                .context("failed to get task-comment")?,
        ))
    }
    pub async fn create(&self, task_comment: TaskComment) -> anyhow::Result<Uuid> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        self.storage
            .task_comments()
            .create(
                &mut conn.as_mut(),
                mapper::task_comment_uc_to_task_comment_db(task_comment),
            )
            .await
            .context("failed to create task-comment")
    }
    pub async fn delete(&self, item_id: Uuid) -> anyhow::Result<()> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        self.storage
            .task_comments()
            .delete(&mut conn.as_mut(), item_id)
            .await
            .context("failed to delete task-comment")
    }
}
