use http::StatusCode;
use uuid::Uuid;

use crate::adapter::db::RepositoryError;
use crate::adapter::db::postgres::tables::task_comments::TaskComments as TaskCommentsRepo;
use crate::err_msg::ErrMsg;

use super::{UseCaseError, mapper, models::TaskComment};

#[derive(Clone)] // из-за axum-state
pub struct TaskComments {
    task_comments_repo: TaskCommentsRepo,
}

impl TaskComments {
    pub fn new(task_comments_repo: TaskCommentsRepo) -> Self {
        Self { task_comments_repo }
    }
    pub async fn list(
        &self,
        task_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<TaskComment>, i64), UseCaseError> {
        let (items, total) = self
            .task_comments_repo
            .list(task_id, limit, offset)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to get items: {e}")))?;
        Ok((
            items
                .into_iter()
                .map(mapper::task_comment_db_to_task_comment_uc)
                .collect(),
            total,
        ))
    }
    // one. нужно чтоб отдать через create
    pub async fn one(&self, item_id: Uuid) -> Result<TaskComment, UseCaseError> {
        let task_comment_db = self
            .task_comments_repo
            .one(item_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFoundRow => UseCaseError::ForTransport {
                    status_code: StatusCode::NOT_FOUND,
                    public_err: ErrMsg::NotFoundItem.as_str(),
                    internal_err: None,
                },
                other => UseCaseError::Common(other.to_string()),
            })?;
        Ok(mapper::task_comment_db_to_task_comment_uc(task_comment_db))
    }
    pub async fn create(&self, task_comment: TaskComment) -> Result<Uuid, UseCaseError> {
        let new_uuid = self
            .task_comments_repo
            .create(mapper::task_comment_uc_to_task_comment_db(task_comment))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;
        Ok(new_uuid)
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        self.task_comments_repo
            .delete(item_id)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to delete: {e}")))?;
        Ok(())
    }
}
