use http::StatusCode;
use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    postgres::{
        tables::{
            task_histories::TaskHistories as DBTaskHistories, tasks::Status as TaskStatus,
            tasks::Tasks as DBTasks, team_members::TeamMembers as DBTeamMembers,
        },
        transactor::Transactor,
    },
};
use crate::err_msg::ErrMsg;

use super::{
    UseCaseError, mapper,
    models::{Task, TaskData, TaskHistory},
};

#[derive(Clone)] // из-за axum-state
pub struct Tasks {
    transactor: Transactor,
    tasks_repo: DBTasks,
    task_histories_repo: DBTaskHistories,
    team_members_repo: DBTeamMembers,
}

impl Tasks {
    pub fn new(
        transactor: Transactor,
        tasks_repo: DBTasks,
        task_histories_repo: DBTaskHistories,
        team_members_repo: DBTeamMembers,
    ) -> Self {
        Self {
            transactor,
            tasks_repo,
            task_histories_repo,
            team_members_repo,
        }
    }
    pub async fn list(&self, data: TaskData) -> Result<(Vec<Task>, i64), UseCaseError> {
        Ok(self
            .transactor
            .in_transaction::<_, _, UseCaseError>(async |tx| {
                let list = self
                    .tasks_repo
                    .list(tx, mapper::task_data_uc_to_task_data_db(data))
                    .await?;
                Ok((
                    list.0.into_iter().map(mapper::task_db_to_task_uc).collect(),
                    list.1,
                ))
            })
            .await?)
    }
    pub async fn one(&self, item_id: Uuid) -> Result<Task, UseCaseError> {
        let mut db_conn = self.transactor.conn().await?;
        Ok(mapper::task_db_to_task_uc(
            self.tasks_repo.one(&mut db_conn, &item_id).await?,
        ))
    }
    pub async fn create(&self, task: Task, user_id: Uuid) -> Result<Uuid, UseCaseError> {
        // создать задачу может только член команды
        self.check_access(task.team_id, user_id).await?;

        Ok(self
            .transactor
            .in_transaction::<_, _, UseCaseError>(async |tx| {
                let new_task_uuid = self
                    .tasks_repo
                    .create(tx.as_mut(), mapper::task_uc_to_task_db(task))
                    .await?;
                let _ = self
                    .task_histories_repo
                    .create(
                        tx,
                        mapper::task_history_uc_to_task_history_db(TaskHistory {
                            task_history_id: Default::default(),
                            task_id: new_task_uuid,
                            user_id,
                            msg: "create".to_string(),
                            created_at: Default::default(),
                        }),
                    )
                    .await?;

                Ok(new_task_uuid)
            })
            .await?)
    }
    pub async fn update(&self, task: Task, user_id: Uuid) -> Result<(), UseCaseError> {
        // обновить задачу может только член команды
        self.check_access(task.team_id, user_id).await?;

        let task_id = task.task_id; // copy-semantic
        let _ = self
            .transactor
            .in_transaction(async |tx| {
                self.tasks_repo
                    .update(tx.as_mut(), mapper::task_uc_to_task_db(task))
                    .await?;

                self.task_histories_repo
                    .create(
                        tx,
                        mapper::task_history_uc_to_task_history_db(TaskHistory {
                            task_history_id: Default::default(),
                            task_id,
                            user_id,
                            msg: "update".to_string(),
                            created_at: Default::default(),
                        }),
                    )
                    .await
            })
            .await?;

        Ok(())
    }
    // удалить задачу может только член команды
    pub async fn delete(&self, task_id: Uuid, user_id: Uuid) -> Result<(), UseCaseError> {
        let mut db_conn = self.transactor.conn().await?;
        let mut task =
            mapper::task_db_to_task_uc(self.tasks_repo.one(&mut db_conn, &task_id).await?);

        self.check_access(task.team_id, user_id).await?;
        task.status = TaskStatus::Cancelled.to_string();

        let _ = self
            .transactor
            .in_transaction(async |tx| {
                self.tasks_repo
                    .update(tx.as_mut(), mapper::task_uc_to_task_db(task))
                    .await?;

                self.task_histories_repo
                    .create(
                        tx,
                        mapper::task_history_uc_to_task_history_db(TaskHistory {
                            task_history_id: Default::default(),
                            task_id,
                            user_id,
                            msg: "delete".to_string(),
                            created_at: Default::default(),
                        }),
                    )
                    .await
            })
            .await?;

        Ok(())
    }
    pub async fn get_history(&self, item_id: Uuid) -> Result<Vec<TaskHistory>, UseCaseError> {
        let mut db_conn = self.transactor.conn().await?;
        Ok(self
            .task_histories_repo
            .by_task_id(&mut db_conn, &item_id)
            .await?
            .into_iter() // по значениям
            .map(mapper::task_history_db_to_task_history_uc)
            .collect())
    }
    async fn check_access(&self, team_id: Uuid, user_id: Uuid) -> Result<(), UseCaseError> {
        let mut db_conn = self.transactor.conn().await?;
        self.team_members_repo
            .one(&mut db_conn, &team_id, &user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFoundRow => UseCaseError::Transport {
                    status_code: StatusCode::FORBIDDEN,
                    public_err: ErrMsg::NoAccessTeamMemberOnly.to_string(),
                    internal_err: None,
                },
                other => UseCaseError::Common(other.to_string()),
            })
            .map(|_| ())
    }
}
