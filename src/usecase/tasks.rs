use http::StatusCode;
use uuid::Uuid;

use crate::{
    adapter::{
        db::RepositoryError,
        db::postgres::tables::{
            task_histories::TaskHistories as TaskHistoriesRepo, tasks::Tasks as TasksRepo,
            team_members::TeamMembers as TeamMembersRepo,
        },
    },
    err_msg::ErrMsg,
};

use super::{
    UseCaseError, mapper,
    models::{Task, TaskHistory},
};

#[derive(Clone)] // из-за axum-state
pub struct Tasks {
    tasks_repo: TasksRepo,
    task_histories_repo: TaskHistoriesRepo,
    team_members_repo: TeamMembersRepo,
}

impl Tasks {
    pub fn new(
        tasks_repo: TasksRepo,
        task_histories_repo: TaskHistoriesRepo,
        team_members_repo: TeamMembersRepo,
    ) -> Self {
        Self {
            tasks_repo,
            task_histories_repo,
            team_members_repo,
        }
    }
    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Task>, i64), UseCaseError> {
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
                public_err: ErrMsg::NotFoundItem.as_str(),
                internal_err: None,
            },
            other => UseCaseError::Common(other.to_string()),
        })?;
        Ok(mapper::task_db_to_task_uc(task_db))
    }
    // создать задачу может только член команды
    pub async fn create(&self, task: Task, user_id: Uuid) -> Result<Uuid, UseCaseError> {
        self.check_access_for_team_member_only(task.team_id, user_id)
            .await
            .map_err(|e| e)?;
        let new_uuid = self
            .tasks_repo
            .create(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to create: {e}")))?;
        Ok(new_uuid)
    }
    // изменить задачу может только член команды
    pub async fn update(&self, task: Task, user_id: Uuid) -> Result<(), UseCaseError> {
        // обновить задачу может только член команды
        self.check_access_for_team_member_only(task.team_id, user_id)
            .await
            .map_err(|e| e)?;
        self.tasks_repo
            .update(mapper::task_uc_to_task_db(task))
            .await
            .map_err(|e| UseCaseError::Common(format!("failed to update: {e}")))?;
        Ok(())
    }
    // удалить задачу может только член команды
    pub async fn delete(&self, task_id: Uuid, user_id: Uuid) -> Result<(), UseCaseError> {
        let task = self.tasks_repo.one(task_id).await.map_err(|e| match e {
            RepositoryError::NotFoundRow => UseCaseError::ForTransport {
                status_code: StatusCode::NOT_FOUND,
                public_err: ErrMsg::NotFoundItem.as_str(),
                internal_err: None,
            },
            other => UseCaseError::Common(other.to_string()),
        })?;
        self.check_access_for_team_member_only(task.team_id, user_id)
            .await
            .map_err(|e| e)?;
        self.tasks_repo
            .delete(task_id)
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
                    public_err: ErrMsg::NoAccessTeamMemberOnly.as_str(),
                    internal_err: None,
                },
                other => UseCaseError::Common(other.to_string()),
            })?;
        Ok(())
    }
}
