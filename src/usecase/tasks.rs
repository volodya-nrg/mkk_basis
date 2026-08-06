use super::{mapper, models::*};
// use crate::adapter::db::errors::Error as DBError;
use crate::adapter::db::postgres::tables::{
    task_histories::TaskHistories as TaskHistoriesRepo, tasks::Tasks as TasksRepo,
};
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
    pub async fn get_list(&self, limit: i32, offset: i32) -> Result<(Vec<Task>, i64), String> {
        let (items, total) = self
            .tasks_repo
            .list(limit, offset)
            .await
            .map_err(|e| format!("failed to get items: {e}"))?;
        Ok((
            items
                .into_iter()
                .map(|item| mapper::task_db_to_task_uc(item))
                .collect(),
            total,
        ))
    }
    pub async fn create(&self, task: Task) -> Result<Uuid, String> {
        let new_uuid = self
            .tasks_repo
            .create(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| format!("failed to create: {e}"))?;
        Ok(new_uuid)
    }
    pub async fn update(&self, task: Task) -> Result<(), String> {
        self.tasks_repo
            .update(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| format!("failed to update: {e}"))?;
        Ok(())
    }
    pub async fn get_history(&self, item_id: Uuid) -> Result<Vec<TaskHistory>, String> {
        let items = self
            .task_histories_repo
            .get_by_task_id(item_id)
            .await
            .map_err(|e| format!("failed to get items: {e}"))?;
        Ok(items
            .into_iter()
            .map(|item| mapper::task_history_db_to_task_history_uc(item))
            .collect())
    }
    // pub async fn get_one(&self, item_id: Uuid) -> Result<Task, String> {
    //     let result = self.tasks_repo.one(item_id).await;
    //     let item = match result {
    //         Ok(v) => v,
    //         Err(e) => {
    //             return match e {
    //                 DBError::NotFound => Err(format!("not found item({})", item_id)),
    //                 DBError::Any(e) => Err(format!("failed to get item: {e}")),
    //             };
    //         }
    //     };
    //
    //     Ok(mapper::task_db_to_task_uc(item))
    // }
}
