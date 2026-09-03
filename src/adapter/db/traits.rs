use uuid::Uuid;

use crate::adapter::db::{
    errors::RepositoryError,
    models::{Task, TaskComment, TaskData, TaskHistory, Team, TeamMember, User},
};

pub trait DBInterface: Clone + Send + Sync + 'static {
    // Clone для клонирования usecase-а в роутере в state
    type UsersRepo: UsersInterface;
    type TeamsRepo: TeamsInterface;
    type TasksRepo: TasksInterface;
    type TaskHistoriesRepo: TaskHistoriesInterface;
    type TeamMembersRepo: TeamMembersInterface;
    type TaskCommentsRepo: TaskCommentsInterface;

    fn users(&self) -> &Self::UsersRepo;
    fn teams(&self) -> &Self::TeamsRepo;
    fn tasks(&self) -> &Self::TasksRepo;
    fn task_histories(&self) -> &Self::TaskHistoriesRepo;
    fn team_members(&self) -> &Self::TeamMembersRepo;
    fn task_comments(&self) -> &Self::TaskCommentsRepo;
}

pub trait TaskCommentsInterface: Clone + Send + Sync + 'static {
    async fn list(
        &self,
        task_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<TaskComment>, i64), RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<TaskComment, RepositoryError>;
    async fn create(&self, item: TaskComment) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: TaskComment) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}

pub trait TaskHistoriesInterface: Clone + Send + Sync + 'static {
    async fn list(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<TaskHistory>, i64), RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<TaskHistory, RepositoryError>;
    async fn by_task_id(&self, task_id: Uuid) -> Result<Vec<TaskHistory>, RepositoryError>;
    async fn create(&self, item: TaskHistory) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: TaskHistory) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}

pub trait TasksInterface: Clone + Send + Sync + 'static {
    async fn list(&self, data: TaskData) -> Result<(Vec<Task>, i64), RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<Task, RepositoryError>;
    async fn create(&self, item: Task) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: Task) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}

pub trait TeamMembersInterface: Clone + Send + Sync + 'static {
    async fn all(&self) -> Result<Vec<TeamMember>, RepositoryError>;
    async fn one(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMember, RepositoryError>;
    async fn create(&self, item: TeamMember) -> Result<(), RepositoryError>;
    async fn delete(&self, team_id: Uuid, user_id: Uuid) -> Result<(), RepositoryError>;
}

pub trait TeamsInterface: Clone + Send + Sync + 'static {
    async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Team>, i64), RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<Team, RepositoryError>;
    async fn create(&self, item: Team) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: Team) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}

// UsersRepoInterface - структуре Users обязательно нужно реализовать данный трейт, чтоб после все
// кто использует данный трейт видели Users.
// Clone - соотв-но для возможности клонирования.
pub trait UsersInterface: Clone + Send + Sync + 'static {
    // : Clone + Send + Sync + 'static
    async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<User>, i64), RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<User, RepositoryError>;
    async fn by_email(&self, email: String) -> Result<User, RepositoryError>;
    async fn create(&self, item: User) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: User) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}
