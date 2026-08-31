use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub email_code: Option<String>,
    pub avatar: Option<String>,
    pub role: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UserCreate {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub email_code: Option<String>,
    pub role: Option<String>,
    pub avatar: Option<String>,
}

pub struct UserUpdate {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    pub email_code: Option<String>,
    pub role: Option<String>,
    pub avatar: Option<String>,
    pub is_remove_avatar: bool,
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

pub struct TaskLimitOffsetFilter {
    pub limit: i32,
    pub offset: i32,
    pub team_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub status: Option<String>,
}
