use chrono::{DateTime, Utc};
use fake::Dummy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Dummy, PartialEq)]
pub struct RequestRegister {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub is_agree: bool,
}

#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct RequestLogin {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestLimitOffsetFilter {
    pub limit: i32,
    pub offset: i32,
    pub filter: String,
}

#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct RequestTeamCreate {
    pub name: String,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct RequestTeamInvite {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct RequestTask {
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub team_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub status: String,
}

// ------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseLogin {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTeamsList {
    pub items: Vec<ResponseTeam>,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTeam {
    pub team_id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTasksList {
    pub items: Vec<ResponseTask>,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTaskHistories {
    pub items: Vec<ResponseTaskHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTaskHistory {
    pub task_history_id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub msg: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseUUID {
    pub uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub message: String,
}
