use anyhow::Context;
use http::StatusCode;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError, postgres::tables::tasks::Status as TaskStatus, storage::Storage,
};
use crate::app_errors::AppErr;

use super::{
    errors::UseCaseError,
    mapper,
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
    pub async fn list(&self, data: TaskData) -> anyhow::Result<(Vec<Task>, i64)> {
        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;
        let list = self
            .storage
            .tasks()
            .list(&mut conn, mapper::task_data_uc_to_task_data_db(data))
            .await
            .context("failed to get list tasks")?;

        tx.commit().await.context("failed to tx-commit")?;
        Ok((
            list.0.into_iter().map(mapper::task_db_to_task_uc).collect(),
            list.1,
        ))
    }
    pub async fn one(&self, item_id: Uuid) -> anyhow::Result<Task> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        Ok(mapper::task_db_to_task_uc(
            self.storage
                .tasks()
                .one(&mut conn.as_mut(), item_id)
                .await
                .map_err(UseCaseError::from)
                .context("failed to get task")?,
        ))
    }
    pub async fn create(&self, task: Task, user_id: Uuid) -> anyhow::Result<Uuid> {
        // создать задачу может только член команды
        self.check_access(task.team_id, user_id)
            .await
            .context("failed to check access")?;

        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;
        let new_task_uuid = self
            .storage
            .tasks()
            .create(&mut conn, mapper::task_uc_to_task_db(task))
            .await
            .context("failed to create task")?;
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
            .await
            .context("failed to create task-history")?;

        tx.commit().await.context("failed to tx-commit")?;
        Ok(new_task_uuid)
    }
    pub async fn update(&self, task: Task, user_id: Uuid) -> anyhow::Result<()> {
        // обновить задачу может только член команды
        self.check_access(task.team_id, user_id)
            .await
            .context("failed to check access")?;

        let task_id = task.task_id;
        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;

        self.storage
            .tasks()
            .update(&mut conn, mapper::task_uc_to_task_db(task))
            .await
            .context("failed to update task")?;
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
            .await
            .context("failed to create task-history")?;

        tx.commit().await.context("failed to tx-commit")
    }
    // удалить задачу может только член команды
    pub async fn delete(&self, task_id: Uuid, user_id: Uuid) -> anyhow::Result<()> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;
        let mut task = mapper::task_db_to_task_uc(
            self.storage
                .tasks()
                .one(&mut conn.as_mut(), task_id)
                .await
                .map_err(UseCaseError::from)
                .context("failed to get task")?,
        );

        self.check_access(task.team_id, user_id)
            .await
            .context("failed to check access")?;
        task.status = TaskStatus::Cancelled.to_string();

        let mut tx = self
            .storage
            .begin()
            .await
            .context("failed to create tx-begin")?;
        let mut conn = tx.get_conn().await.context("failed to get tx-conn")?;

        self.storage
            .tasks()
            .update(&mut conn, mapper::task_uc_to_task_db(task))
            .await
            .context("failed to update task")?;
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
            .await
            .context("failed to create task-history")?;

        tx.commit().await.context("failed to tx-commit")
    }
    pub async fn get_history(&self, item_id: Uuid) -> anyhow::Result<Vec<TaskHistory>> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

        Ok(self
            .storage
            .task_histories()
            .by_task_id(&mut conn.as_mut(), item_id)
            .await
            .context("failed to get task-history")?
            .into_iter() // по значениям
            .map(mapper::task_history_db_to_task_history_uc)
            .collect())
    }
    async fn check_access(&self, team_id: Uuid, user_id: Uuid) -> anyhow::Result<()> {
        let mut conn = self
            .storage
            .get_conn()
            .await
            .context("failed to get db-conn")?;

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
                other => UseCaseError::Internal(anyhow::Error::new(other)),
            })
            .context("failed to get team-member")?;

        Ok(())
    }
}
