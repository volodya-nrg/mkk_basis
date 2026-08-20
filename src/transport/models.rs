use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RequestRegister {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub agreement: bool,
    pub privacy_policy: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestLogin {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestLimitOffset {
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestTeamCreate {
    pub name: String,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestTeamInvite {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestTask {
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub team_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestUser {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub email_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestRegisterConfirm {
    pub email: Option<String>,
    pub code: Option<String>,
}

// ------------------------------------

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseLogin {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseTeamsList {
    pub items: Vec<ResponseTeam>,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseTeam {
    pub team_id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseTasksList {
    pub items: Vec<ResponseTask>,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseTask {
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseTaskHistories {
    pub items: Vec<ResponseTaskHistory>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseTaskHistory {
    pub task_history_id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub msg: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseUUID {
    pub uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseUser {
    pub user_id: Uuid,
    pub name: Option<String>,
    pub email: String,
    pub email_code: Option<String>,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseUsersList {
    pub items: Vec<ResponseUser>,
    pub total: u32,
}