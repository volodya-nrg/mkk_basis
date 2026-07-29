use super::AppState;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;
use std::sync::Arc;

pub struct Handlers {}

impl Handlers {
    pub async fn login(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        println!("--- handler login");
        if let Err(e) = state.use_case.auth.login() {
            println!("{e}")
        }
        Json(json!({
            "status": "ok",
            "message": "login",
        }))
    }
    pub async fn register(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        println!("--- handler register");
        if let Err(e) = state.use_case.auth.register() {
            println!("{e}")
        }
        Json(json!({
            "status": "ok",
            "message": "register",
        }))
    }
    pub async fn tasks(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        println!("--- handler tasks");
        if let Err(e) = state.use_case.tasks.get_one() {
            println!("{e}")
        }
        Json(json!({
            "status": "ok",
            "message": "tasks",
        }))
    }
    pub async fn teams(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        println!("--- handler teams");
        if let Err(e) = state.use_case.teams.get_one() {
            println!("{e}")
        }
        Json(json!({
            "status": "ok",
            "message": "teams",
        }))
    }
}
