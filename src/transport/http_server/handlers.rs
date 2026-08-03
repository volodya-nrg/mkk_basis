use super::AppState;
use crate::transport::models::{RequestRegister, ResponseRegister};
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;
use std::sync::Arc;

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
    pub async fn login(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        // if let Err(e) = state.use_case.auth.login() {
        //     println!("{e}")
        // }
        Json(json!({
            "status": "ok",
            "message": "",
        }))
    }

    // teams
    pub async fn teams_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        // if let Err(e) = state.use_case.teams.get_one() {
        //     println!("{e}")
        // }
        Json(json!({
            "status": "ok",
            "message": "",
        }))
    }
    pub async fn teams_create(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        // if let Err(e) = state.use_case.teams.get_one() {
        //     println!("{e}")
        // }
        Json(json!({
            "status": "ok",
            "message": "",
        }))
    }
    pub async fn teams_invite(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        // if let Err(e) = state.use_case.teams.get_one() {
        //     println!("{e}")
        // }
        Json(json!({
            "status": "ok",
            "message": "",
        }))
    }

    // tasks
    pub async fn tasks_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        // if let Err(e) = state.use_case.tasks.get_one() {
        //     println!("{e}")
        // }
        Json(json!({
            "status": "ok",
            "message": "",
        }))
    }
    pub async fn tasks_create(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        // if let Err(e) = state.use_case.tasks.get_one() {
        //     println!("{e}")
        // }
        Json(json!({
            "status": "ok",
            "message": "",
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
