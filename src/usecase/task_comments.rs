use uuid::Uuid;

use crate::adapter::db::postgres::{
    tables::task_comments::TaskComments as DBTaskComments,
    transactor::{TransactionError, Transactor},
};

use super::{UseCaseError, mapper, models::TaskComment};

#[derive(Clone)] // из-за axum-state
pub struct TaskComments {
    transactor: Transactor,
    task_comments_repo: DBTaskComments,
}

impl TaskComments {
    pub fn new(transactor: Transactor, task_comments_repo: DBTaskComments) -> Self {
        Self {
            transactor,
            task_comments_repo,
        }
    }
    pub async fn list(
        &self,
        task_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<TaskComment>, i64), UseCaseError> {
        self.transactor
            .in_transaction(async |tx| {
                self.task_comments_repo
                    .list(tx, task_id, limit, offset)
                    .await
                    .map_err(|e| UseCaseError::Common(format!("failed to get items: {e}")))
                    .map(|list| {
                        (
                            list.0
                                .into_iter() // по значениям
                                .map(mapper::task_comment_db_to_task_comment_uc)
                                .collect(),
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
    pub async fn one(&self, item_id: Uuid) -> Result<TaskComment, UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        Ok(mapper::task_comment_db_to_task_comment_uc(
            self.task_comments_repo.one(&mut db_conn, item_id).await?,
        ))
    }
    pub async fn create(&self, task_comment: TaskComment) -> Result<Uuid, UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        Ok(self
            .task_comments_repo
            .create(
                &mut db_conn,
                mapper::task_comment_uc_to_task_comment_db(task_comment),
            )
            .await?)
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        let mut db_conn = self
            .transactor
            .conn()
            .await
            .map_err(|e| UseCaseError::Common(e.to_string()))?;
        Ok(self
            .task_comments_repo
            .delete(&mut db_conn, item_id)
            .await?)
    }
}
