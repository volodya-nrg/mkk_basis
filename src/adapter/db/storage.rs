use uuid::Uuid;

use super::errors::RepositoryError;
use super::models::{List, Task, TaskComment, TaskData, TaskHistory, Team, TeamMember, User};

trait Storage {
    type Transaction<'a>: StorageTransaction<'a>
    where
        Self: 'a;
    type TaskComments: TaskCommentsTable;
    type TaskHistories: TaskHistoriesTable;
    type Tasks: TasksTable;
    type TeamMembers: TeamMembersTable;
    type Teams: TeamsTable;
    type Users: UsersTable;

    fn begin(&self) -> Result<Self::Transaction<'_>, RepositoryError>;
    fn task_comments(&self) -> &Self::TaskComments;
    fn task_histories(&self) -> &Self::TaskHistories;
    fn tasks(&self) -> &Self::Tasks;
    fn team_members(&self) -> &Self::TeamMembers;
    fn teams(&self) -> &Self::Teams;
    fn users(&self) -> &Self::Users;
}

trait StorageTransaction<'a> {
    /*
    let tx = storage.begin()?;
    // работаем через tx, у которого те же геттеры таблиц
    tx.tasks().create(...)?;
    tx.commit()?;
    */
    fn commit(&self) -> Result<(), RepositoryError>;
    fn rollback(&self) -> Result<(), RepositoryError>;
}

trait TaskCommentsTable {
    async fn list(
        &self,
        task_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> Result<List<TaskComment>, RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<TaskComment, RepositoryError>;
    async fn create(&self, item: TaskComment) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: TaskComment) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}
trait TaskHistoriesTable {
    async fn list(&self, limit: i32, offset: i32) -> Result<List<TaskHistory>, RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<TaskHistory, RepositoryError>;
    async fn by_task_id(&self, task_id: Uuid) -> Result<Vec<TaskHistory>, RepositoryError>;
    async fn create(&self, item: TaskHistory) -> Result<TaskHistory, RepositoryError>;
    async fn update(&self, item: TaskHistory) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}
trait TasksTable {
    async fn list(&self, data: TaskData) -> Result<List<Task>, RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<Task, RepositoryError>;
    async fn create(&self, item: Task) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: Task) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}
trait TeamMembersTable {
    async fn all(&self) -> Result<Vec<TeamMember>, RepositoryError>;
    async fn one(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMember, RepositoryError>;
    async fn create(&self, item: TeamMember) -> Result<(), RepositoryError>;
    async fn delete(&self, team_id: Uuid, user_id: Uuid) -> Result<(), RepositoryError>;
}
trait TeamsTable {
    async fn list(&self, limit: i32, offset: i32) -> Result<List<Team>, RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<Team, RepositoryError>;
    async fn create(&self, item: Team) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: Team) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}
trait UsersTable {
    async fn list(&self, limit: i32, offset: i32) -> Result<List<User>, RepositoryError>;
    async fn one(&self, item_id: Uuid) -> Result<User, RepositoryError>;
    async fn by_email(&self, email: &str) -> Result<User, RepositoryError>;
    async fn create(&self, item: User) -> Result<Uuid, RepositoryError>;
    async fn update(&self, item: User) -> Result<(), RepositoryError>;
    async fn delete(&self, item_id: Uuid) -> Result<(), RepositoryError>;
}
