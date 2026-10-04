use async_trait::async_trait;
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres, Sqlite, SqliteConnection};
use uuid::Uuid;

use super::{
    errors::RepositoryError,
    models::{List, Task, TaskComment, TaskData, TaskHistory, Team, TeamMember, User},
};

// Заимствованное соединение — для передачи в методы таблиц.
pub enum AnyConnection<'a> {
    Postgres(&'a mut PgConnection),
    Sqlite(&'a mut SqliteConnection),
}

// Владеющее соединение — то, что возвращает Storage::get_conn().
pub enum AnyConnectionOwned {
    Postgres(PoolConnection<Postgres>),
    Sqlite(PoolConnection<Sqlite>),
}

impl AnyConnectionOwned {
    pub fn as_mut(&mut self) -> AnyConnection<'_> {
        match self {
            Self::Postgres(c) => AnyConnection::Postgres(&mut *c),
            Self::Sqlite(c) => AnyConnection::Sqlite(&mut *c),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    // ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    pub const fn as_sql(&self) -> &'static str {
        match self {
            // Self::ReadUncommitted => "READ UNCOMMITTED",
            Self::ReadCommitted => "READ COMMITTED",
            Self::RepeatableRead => "REPEATABLE READ",
            Self::Serializable => "SERIALIZABLE",
        }
    }
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn MyTransaction>, RepositoryError>;
    async fn get_conn(&self) -> Result<AnyConnectionOwned, RepositoryError>;
    fn task_comments(&self) -> &dyn TaskCommentsTable;
    fn task_histories(&self) -> &dyn TaskHistoriesTable;
    fn tasks(&self) -> &dyn TasksTable;
    fn team_members(&self) -> &dyn TeamMembersTable;
    fn teams(&self) -> &dyn TeamsTable;
    fn users(&self) -> &dyn UsersTable;
}
#[async_trait]
pub trait MyTransaction: Send + Sync {
    async fn get_conn(&mut self) -> Result<AnyConnection<'_>, RepositoryError>;
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
    async fn rollback(self: Box<Self>) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TaskCommentsTable: Send + Sync {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        task_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<List<TaskComment>, RepositoryError>;

    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<TaskComment, RepositoryError>;
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TaskComment,
    ) -> Result<Uuid, RepositoryError>;
    async fn update(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TaskComment,
    ) -> Result<(), RepositoryError>;
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TaskHistoriesTable: Send + Sync {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        limit: i32,
        offset: i32,
    ) -> Result<List<TaskHistory>, RepositoryError>;
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<TaskHistory, RepositoryError>;
    async fn by_task_id(
        &self,
        conn: &mut AnyConnection<'_>,
        task_id: Uuid,
    ) -> Result<Vec<TaskHistory>, RepositoryError>;
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TaskHistory,
    ) -> Result<Uuid, RepositoryError>;
    async fn update(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TaskHistory,
    ) -> Result<(), RepositoryError>;
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TasksTable: Send + Sync {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        data: TaskData,
    ) -> Result<List<Task>, RepositoryError>;
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<Task, RepositoryError>;
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: Task,
    ) -> Result<Uuid, RepositoryError>;
    async fn update(&self, conn: &mut AnyConnection<'_>, item: Task)
    -> Result<(), RepositoryError>;
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TeamMembersTable: Send + Sync {
    async fn all(&self, conn: &mut AnyConnection<'_>) -> Result<Vec<TeamMember>, RepositoryError>;
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<TeamMember, RepositoryError>;
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: TeamMember,
    ) -> Result<(), RepositoryError>;
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        team_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TeamsTable: Send + Sync {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        limit: i32,
        offset: i32,
    ) -> Result<List<Team>, RepositoryError>;
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<Team, RepositoryError>;
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: Team,
    ) -> Result<Uuid, RepositoryError>;
    async fn update(&self, conn: &mut AnyConnection<'_>, item: Team)
    -> Result<(), RepositoryError>;
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait UsersTable: Send + Sync {
    async fn list(
        &self,
        conn: &mut AnyConnection<'_>,
        limit: i32,
        offset: i32,
    ) -> Result<List<User>, RepositoryError>;
    async fn one(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<User, RepositoryError>;
    async fn by_email(
        &self,
        conn: &mut AnyConnection<'_>,
        email: &str,
    ) -> Result<User, RepositoryError>;
    async fn create(
        &self,
        conn: &mut AnyConnection<'_>,
        item: User,
    ) -> Result<Uuid, RepositoryError>;
    async fn update(&self, conn: &mut AnyConnection<'_>, item: User)
    -> Result<(), RepositoryError>;
    async fn delete(
        &self,
        conn: &mut AnyConnection<'_>,
        item_id: Uuid,
    ) -> Result<(), RepositoryError>;
}
