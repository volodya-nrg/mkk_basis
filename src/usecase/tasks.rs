use super::{
    UseCaseError, mapper,
    models::{Task, TaskHistory},
};
use crate::adapter::{
    db::RepositoryError,
    db::postgres::tables::{
        task_histories::TaskHistories as TaskHistoriesRepo, tasks::Tasks as TasksRepo,
    },
};
use http::StatusCode;
use uuid::Uuid;

#[derive(Clone)] // из-за axum-state
pub struct Tasks {
    tasks_repo: TasksRepo,
    task_histories_repo: TaskHistoriesRepo,
}

impl Tasks {
    pub fn new(tasks_repo: TasksRepo, task_histories_repo: TaskHistoriesRepo) -> Self {
        Self {
            tasks_repo,
            task_histories_repo,
        }
    }
    pub async fn list(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<Task>, i64), UseCaseError> {
        let (items, total) = self
            .tasks_repo
            .list(limit, offset)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to get items: {e}")))?;
        Ok((
            items.into_iter().map(mapper::task_db_to_task_uc).collect(),
            total,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Task, UseCaseError> {
        let task_db = self.tasks_repo.one(item_id).await.map_err(|e| match e {
            RepositoryError::NotFoundRow => UseCaseError::ForTransport {
                status_code: StatusCode::NOT_FOUND,
                public_err: "item not found".to_string(),
                internal_err: None,
            },
            other => UseCaseError::Common(other.to_string()),
        })?;
        Ok(mapper::task_db_to_task_uc(task_db))
    }
    pub async fn create(&self, task: Task) -> Result<Uuid, UseCaseError> {
        let new_uuid = self
            .tasks_repo
            .create(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;
        Ok(new_uuid)
    }
    pub async fn update(&self, task: Task) -> Result<(), UseCaseError> {
        self.tasks_repo
            .update(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to update: {e}")))?;
        Ok(())
    }
    pub async fn delete(&self, item_id: Uuid) -> Result<(), UseCaseError> {
        self.tasks_repo
            .delete(item_id)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to delete: {e}")))?;
        Ok(())
    }
    pub async fn get_history(&self, item_id: Uuid) -> Result<Vec<TaskHistory>, UseCaseError> {
        let items = self
            .task_histories_repo
            .by_task_id(item_id)
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to get items: {e}")))?;
        Ok(items
            .into_iter()
            .map(mapper::task_history_db_to_task_history_uc)
            .collect())
    }
}
