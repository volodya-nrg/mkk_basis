use chrono::{DateTime, Utc};
use fake::Dummy;
use fake::faker::internet::raw::{FreeEmail, Password};
use fake::locales::EN;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Dummy, PartialEq, Clone)]
pub struct RequestRegister {
    #[dummy(faker = "FreeEmail(EN)")]
    pub email: String,
    #[dummy(faker = "Password(EN, 5..20)")]
    pub password: String,
    #[dummy(faker = "Password(EN, 5..20)")]
    pub password_confirm: String,
}

#[derive(Debug, Serialize, Deserialize, Dummy, Clone)]
pub struct RequestLogin {
    #[dummy(faker = "FreeEmail(EN)")]
    pub email: String,
    #[dummy(faker = "Password(EN, 5..20)")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestLimitOffset {
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize, Deserialize, Dummy, Clone)]
pub struct RequestTeamCreate {
    pub name: String,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Dummy, Clone)]
pub struct RequestTeamInvite {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Dummy, Clone)]
pub struct RequestTask {
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub team_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub status: String,
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
