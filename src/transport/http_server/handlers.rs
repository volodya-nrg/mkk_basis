use super::AppState;
use crate::transport::models::*;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub struct Handlers {}

impl Handlers {
    // auth
    pub async fn register(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<RequestRegister>,
    ) -> impl IntoResponse {
        // if let Err(e) = state.use_case.auth.register() {
        //     println!("{e}")
        // }
        Json(json!(ResponseRegister {
            email: payload.email,
            password: payload.password,
            password_confirm: payload.password_confirm,
            is_agree: payload.is_agree,
        }))
    }
    pub async fn login(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse {
        // if let Err(e) = state.use_case.auth.register() {
        //     println!("{e}")
        // }
        Json(json!(ResponseLogin {
            access_token: payload.email.to_string(),
            refresh_token: payload.password.to_string(),
        }))
    }

    // teams
    pub async fn teams_list(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<RequestLimitOffsetFilter>,
    ) -> impl IntoResponse {
        // if let Err(e) = state.use_case.teams.get_one() {
        //     println!("{e}")
        // }
        Json(json!(ResponseTeamsList {
            item: vec![payload.filter],
            page_number: payload.limit.cast_unsigned(),
            total: payload.offset.into(),
        }))
    }
    pub async fn teams_create(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<RequestTeamCreate>,
    ) -> impl IntoResponse {
        // if let Err(e) = state.use_case.teams.get_one() {
        //     println!("{e}")
        // }
        Json(json!(ResponseTeam {
            team_id: Uuid::new_v4(),
            name: payload.name.to_string(),
            created_by: payload.created_by,
            created_at: Default::default(),
            updated_at: Default::default(),
        }))
    }
    pub async fn teams_invite(
        State(state): State<Arc<AppState>>,
        Path(team_id): Path<Uuid>,
        Json(payload): Json<RequestTeamInvite>,
    ) -> impl IntoResponse {
        // if let Err(e) = state.use_case.teams.get_one() {
        //     println!("{e}")
        // }
        (StatusCode::CREATED, ())
    }

    // tasks
    pub async fn tasks_list(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<RequestLimitOffsetFilter>,
    ) -> impl IntoResponse {
        // if let Err(e) = state.use_case.teams.get_one() {
        //     println!("{e}")
        // }
        Json(json!(ResponseTasksList {
            item: vec![payload.filter],
            page_number: payload.limit.cast_unsigned(),
            total: payload.offset.into(),
        }))
    }
    pub async fn tasks_create(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<RequestTaskCreate>,
    ) -> impl IntoResponse {
        // if let Err(e) = state.use_case.teams.get_one() {
        //     println!("{e}")
        // }
        Json(json!(ResponseTask {
            task_id: Uuid::new_v4(),
            name: payload.name,
            description: payload.description,
            created_by: payload.created_by,
            team_id: payload.team_id,
            assignee_id: payload.assignee_id,
            status: "start".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        }))
    }
    pub async fn tasks_update(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        // if let Err(e) = state.use_case.tasks.get_one() {
        //     println!("{e}")
        // }
        Json(json!({
            "status": "ok",
            "message": "",
        }))
    }
    pub async fn tasks_history(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        // if let Err(e) = state.use_case.tasks.get_one() {
        //     println!("{e}")
        // }
        Json(json!({
            "status": "ok",
            "message": "",
        }))
    }
}
