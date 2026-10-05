pub mod tables;
pub mod transactor;

use async_trait::async_trait;
use sqlx::AssertSqlSafe;
use sqlx::{Pool, Postgres as SQLXPostgres};

use crate::adapter::db::{
    errors::RepositoryError,
    storage::{
        AnyConnectionOwned, IsolationLevel, Storage, TaskCommentsTable, TaskHistoriesTable,
        TasksTable, TeamMembersTable, TeamsTable, Transaction, UsersTable,
    },
};
use tables::{
    task_comments::TaskComments, task_histories::TaskHistories, tasks::Tasks,
    team_members::TeamMembers, teams::Teams, users::Users,
};
use transactor::Transactor;

pub struct Postgres {
    level: IsolationLevel,
    pool: Pool<SQLXPostgres>,
    pub tbl_task_comments: TaskComments,
    pub tbl_task_histories: TaskHistories,
    pub tbl_tasks: Tasks,
    pub tbl_team_members: TeamMembers,
    pub tbl_teams: Teams,
    pub tbl_users: Users,
}

impl Postgres {
    pub const fn new(pool: Pool<SQLXPostgres>, level: IsolationLevel) -> Self {
        Self {
            level,
            pool,
            tbl_task_comments: TaskComments {},
            tbl_task_histories: TaskHistories {},
            tbl_tasks: Tasks {},
            tbl_team_members: TeamMembers {},
            tbl_teams: Teams {},
            tbl_users: Users {},
        }
    }
}

#[async_trait]
impl Storage for Postgres {
    async fn begin(&self) -> Result<Box<dyn Transaction>, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(RepositoryError::Common)?;
        let sql = format!("SET TRANSACTION ISOLATION LEVEL {}", self.level.as_sql());
        sqlx::query(AssertSqlSafe(sql))
            .execute(&mut *tx)
            .await
            .map_err(RepositoryError::Common)?;
        Ok(Box::new(Transactor::new(tx)))
    }
    async fn get_conn(&self) -> Result<AnyConnectionOwned, RepositoryError> {
        let conn = self.pool.acquire().await.map_err(RepositoryError::Common)?;
        Ok(AnyConnectionOwned::Postgres(conn))
    }
    fn task_comments(&self) -> &dyn TaskCommentsTable {
        &self.tbl_task_comments
    }
    fn task_histories(&self) -> &dyn TaskHistoriesTable {
        &self.tbl_task_histories
    }
    fn tasks(&self) -> &dyn TasksTable {
        &self.tbl_tasks
    }
    fn team_members(&self) -> &dyn TeamMembersTable {
        &self.tbl_team_members
    }
    fn teams(&self) -> &dyn TeamsTable {
        &self.tbl_teams
    }
    fn users(&self) -> &dyn UsersTable {
        &self.tbl_users
    }
}
