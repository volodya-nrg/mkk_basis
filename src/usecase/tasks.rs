use http::StatusCode;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError, postgres::tables::tasks::Status as TaskStatus, storage::Storage,
};
use crate::app_errors::AppErr;

use super::{
    UseCaseError, mapper,
    models::{Task, TaskData, TaskHistory},
};

#[derive(Clone)] // clone из-за axum
pub struct Tasks {
    storage: Arc<dyn Storage>,
}

impl Tasks {
    pub const fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }
    pub async fn list(&self, data: TaskData) -> Result<(Vec<Task>, i64), UseCaseError> {
        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;
        let list = self
            .storage
            .tasks()
            .list(&mut conn, mapper::task_data_uc_to_task_data_db(data))
            .await?;

        tx.commit().await?;
        Ok((
            list.0.into_iter().map(mapper::task_db_to_task_uc).collect(),
            list.1,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Task, UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(mapper::task_db_to_task_uc(
            self.storage
                .tasks()
                .one(&mut conn.as_mut(), item_id)
                .await?,
        ))
    }
    pub async fn create(&self, task: Task, user_id: Uuid) -> Result<Uuid, UseCaseError> {
        // создать задачу может только член команды
        self.check_access(task.team_id, user_id).await?;

        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;
        let new_task_uuid = self
            .storage
            .tasks()
            .create(&mut conn, mapper::task_uc_to_task_db(task))
            .await?;
        let _ = self
            .storage
            .task_histories()
            .create(
                &mut conn,
                mapper::task_history_uc_to_task_history_db(TaskHistory {
                    task_history_id: Default::default(),
                    task_id: new_task_uuid,
                    user_id,
                    msg: "create".to_string(),
                    created_at: Default::default(),
                }),
            )
            .await?;

        tx.commit().await?;
        Ok(new_task_uuid)
    }
    pub async fn update(&self, task: Task, user_id: Uuid) -> Result<(), UseCaseError> {
        // обновить задачу может только член команды
        self.check_access(task.team_id, user_id).await?;

        let task_id = task.task_id;
        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;

        self.storage
            .tasks()
            .update(&mut conn, mapper::task_uc_to_task_db(task))
            .await?;
        self.storage
            .task_histories()
            .create(
                &mut conn,
                mapper::task_history_uc_to_task_history_db(TaskHistory {
                    task_history_id: Default::default(),
                    task_id,
                    user_id,
                    msg: "update".to_string(),
                    created_at: Default::default(),
                }),
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }
    // удалить задачу может только член команды
    pub async fn delete(&self, task_id: Uuid, user_id: Uuid) -> Result<(), UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        let mut task = mapper::task_db_to_task_uc(
            self.storage
                .tasks()
                .one(&mut conn.as_mut(), task_id)
                .await?,
        );

        self.check_access(task.team_id, user_id).await?;
        task.status = TaskStatus::Cancelled.to_string();

        let mut tx = self.storage.begin().await?;
        let mut conn = tx.get_conn().await?;

        self.storage
            .tasks()
            .update(&mut conn, mapper::task_uc_to_task_db(task))
            .await?;
        self.storage
            .task_histories()
            .create(
                &mut conn,
                mapper::task_history_uc_to_task_history_db(TaskHistory {
                    task_history_id: Default::default(),
                    task_id,
                    user_id,
                    msg: "delete".to_string(),
                    created_at: Default::default(),
                }),
            )
            .await?;

        tx.commit().await?;
        Ok(())
    }
    pub async fn get_history(&self, item_id: Uuid) -> Result<Vec<TaskHistory>, UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        Ok(self
            .storage
            .task_histories()
            .by_task_id(&mut conn.as_mut(), item_id)
            .await?
            .into_iter() // по значениям
            .map(mapper::task_history_db_to_task_history_uc)
            .collect())
    }
    async fn check_access(&self, team_id: Uuid, user_id: Uuid) -> Result<(), UseCaseError> {
        let mut conn = self.storage.get_conn().await?;
        self.storage
            .team_members()
            .one(&mut conn.as_mut(), team_id, user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFoundRow => UseCaseError::Transport {
                    status_code: StatusCode::FORBIDDEN,
                    public_err: AppErr::NoAccessTeamMemberOnly.to_string(),
                    internal_err: None,
                },
                other => UseCaseError::Common(other.to_string()),
            })
            .map(|_| ())
    }
}
