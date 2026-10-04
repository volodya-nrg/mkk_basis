pub mod tables;
pub mod transactor;

use crate::adapter::db::storage::{
    AnyConnection, AnyConnectionOwned, TaskHistoriesTable, TasksTable, TeamMembersTable,
    TeamsTable, UsersTable,
};
use crate::adapter::db::{
    errors::RepositoryError,
    postgres::tables::{
        task_histories::TaskHistories, tasks::Tasks, team_members::TeamMembers, teams::Teams,
        users::Users,
    },
    storage::{IsolationLevel, MyTransaction, Storage, TaskCommentsTable},
};
use async_trait::async_trait;
use sqlx::{AssertSqlSafe, Transaction};
use sqlx::{Pool, Postgres};
use tables::task_comments::TaskComments;

pub struct MyPostgres {
    level: IsolationLevel,
    pool: Pool<Postgres>,
    pub tbl_task_comments: TaskComments,
    pub tbl_task_histories: TaskHistories,
    pub tbl_tasks: Tasks,
    pub tbl_team_members: TeamMembers,
    pub tbl_teams: Teams,
    pub tbl_users: Users,
}

impl MyPostgres {
    pub async fn new(pool: Pool<Postgres>, level: IsolationLevel) -> Result<Self, RepositoryError> {
        //let conn = pool.acquire().await.map_err(RepositoryError::Common)?;
        Ok(Self {
            level,
            pool,
            // conn,
            tbl_task_comments: TaskComments {},
            tbl_task_histories: TaskHistories {},
            tbl_tasks: Tasks {},
            tbl_team_members: TeamMembers {},
            tbl_teams: Teams {},
            tbl_users: Users {},
        })
    }
}

#[async_trait]
impl Storage for MyPostgres {
    async fn begin(&self) -> Result<Box<dyn MyTransaction>, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(RepositoryError::Common)?;
        let sql = format!("SET TRANSACTION ISOLATION LEVEL {}", self.level.as_sql());
        sqlx::query(AssertSqlSafe(sql))
            .execute(&mut *tx)
            .await
            .map_err(RepositoryError::Common)?;
        Ok(Box::new(PostgresTransaction::new(tx)))
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

pub struct PostgresTransaction {
    pub tx: Transaction<'static, Postgres>,
}

impl PostgresTransaction {
    pub const fn new(tx: Transaction<'static, Postgres>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl MyTransaction for PostgresTransaction {
    async fn get_conn(&mut self) -> Result<AnyConnection<'_>, RepositoryError> {
        Ok(AnyConnection::Postgres(&mut self.tx))
    }
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        self.tx
            // .borrow()
            .commit()
            .await
            .map_err(RepositoryError::Common)
        // match self.tx {
        //     AnyTransaction::Postgres(tx) => tx.commit().await.map_err(RepositoryError::Common),
        //     AnyTransaction::Sqlite(tx) => tx.commit().await.map_err(RepositoryError::Common),
        // }
    }
    async fn rollback(self: Box<Self>) -> Result<(), RepositoryError> {
        self.tx
            // .borrow()
            .rollback()
            .await
            .map_err(RepositoryError::Common)
        // match self.tx {
        //     AnyTransaction::Postgres(tx) => tx.rollback().await.map_err(RepositoryError::Common),
        //     AnyTransaction::Sqlite(tx) => tx.rollback().await.map_err(RepositoryError::Common),
        // }
    }
}
