use chrono::{DateTime, Utc};
use fake::Dummy;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Clone, Dummy, PartialEq)]
pub struct User {
    pub user_id: Uuid,
    pub name: Option<String>,
    pub email: String,
    pub password: String,
    pub email_is_confirmed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub struct Team {
    pub team_id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub struct TeamMember {
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}
pub struct Task {
    pub task_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub team_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub struct TaskHistory {
    pub task_history_id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub msg: String,
    pub created_at: DateTime<Utc>,
}
pub struct TaskComment {
    pub task_comment_id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub msg: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
