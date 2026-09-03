use http::StatusCode;
use uuid::Uuid;

use crate::{
    adapter::db::{
        errors::RepositoryError,
        postgres::tables::task_histories::TaskHistories as DBTaskHistories,
        postgres::tables::tasks::Status as TaskStatus, postgres::tables::tasks::Tasks as DBTasks,
        postgres::tables::team_members::TeamMembers as DBTeamMembers,
    },
    err_msg::ErrMsg,
};

use super::{
    UseCaseError, mapper,
    models::{Task, TaskData, TaskHistory},
};

#[derive(Clone)] // из-за axum-state
pub struct Tasks {
    tasks_repo: DBTasks,
    task_histories_repo: DBTaskHistories,
    team_members_repo: DBTeamMembers,
}

impl Tasks {
    pub fn new(
        tasks_repo: DBTasks,
        task_histories_repo: DBTaskHistories,
        team_members_repo: DBTeamMembers,
    ) -> Self {
        Self {
            tasks_repo,
            task_histories_repo,
            team_members_repo,
        }
    }
    pub async fn list(&self, data: TaskData) -> Result<(Vec<Task>, i64), UseCaseError> {
        let (items, total) = self
            .tasks_repo
            .list(mapper::task_data_uc_to_task_data_db(data))
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
                public_err: ErrMsg::NotFoundItem.to_string(),
                internal_err: None,
            },
            other => UseCaseError::Common(other.to_string()),
        })?;
        Ok(mapper::task_db_to_task_uc(task_db))
    }
    // создать задачу может только член команды
    pub async fn create(&self, task: Task, user_id: Uuid) -> Result<Uuid, UseCaseError> {
        self.check_access_for_team_member_only(task.team_id, user_id)
            .await?;
        // TODO tx
        let new_task_uuid = self
            .tasks_repo
            .create(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create task: {e}")))?;
        let _ = self
            .task_histories_repo
            .create(mapper::task_history_uc_to_task_history_db(TaskHistory {
                task_history_id: Default::default(),
                task_id: new_task_uuid,
                user_id,
                msg: "create".to_string(),
                created_at: Default::default(),
            }))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create task_history: {e}")))?;
        // TODO \tx
        Ok(new_task_uuid)
    }
    // изменить задачу может только член команды
    pub async fn update(&self, task: Task, user_id: Uuid) -> Result<(), UseCaseError> {
        // обновить задачу может только член команды
        self.check_access_for_team_member_only(task.team_id, user_id)
            .await?;

        let task_id = task.task_id;

        // TODO tx
        self.tasks_repo
            .update(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to update: {e}")))?;

        let _ = self
            .task_histories_repo
            .create(mapper::task_history_uc_to_task_history_db(TaskHistory {
                task_history_id: Default::default(),
                task_id,
                user_id,
                msg: "update".to_string(),
                created_at: Default::default(),
            }))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create task_history: {e}")))?;
        // TODO \tx

        Ok(())
    }
    // удалить задачу может только член команды
    pub async fn delete(&self, task_id: Uuid, user_id: Uuid) -> Result<(), UseCaseError> {
        let task_db = self.tasks_repo.one(task_id).await.map_err(|e| match e {
            RepositoryError::NotFoundRow => UseCaseError::ForTransport {
                status_code: StatusCode::NOT_FOUND,
                public_err: ErrMsg::NotFoundItem.to_string(),
                internal_err: None,
            },
            other => UseCaseError::Common(other.to_string()),
        })?;
        let mut task = mapper::task_db_to_task_uc(task_db);
        self.check_access_for_team_member_only(task.team_id, user_id)
            .await?;

        task.status = TaskStatus::Cancelled.to_string();
        // TODO tx
        self.tasks_repo
            .update(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to delete-update: {e}")))?;

        let _ = self
            .task_histories_repo
            .create(mapper::task_history_uc_to_task_history_db(TaskHistory {
                task_history_id: Default::default(),
                task_id,
                user_id,
                msg: "delete".to_string(),
                created_at: Default::default(),
            }))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create task_history: {e}")))?;
        // TODO \tx

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
    async fn check_access_for_team_member_only(
        &self,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), UseCaseError> {
        self.team_members_repo
            .one(team_id, user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFoundRow => UseCaseError::ForTransport {
                    status_code: StatusCode::FORBIDDEN,
                    public_err: ErrMsg::NoAccessTeamMemberOnly.to_string(),
                    internal_err: None,
                },
                other => UseCaseError::Common(other.to_string()),
            })?;
        Ok(())
    }
}
