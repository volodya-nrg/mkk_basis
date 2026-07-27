use crate::usecase::UseCase;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;

pub struct Handlers {
    use_case: UseCase,
}

impl Handlers {
    pub fn new(use_case: UseCase) -> Self {
        Self { use_case }
    }
    pub async fn login() -> impl IntoResponse {
        Json(json!({
            "status": "ok",
            "message": "login",
        }))
    }
    pub async fn register() -> impl IntoResponse {
        Json(json!({
            "status": "ok",
            "message": "register",
        }))
    }
    pub async fn tasks() -> impl IntoResponse {
        Json(json!({
            "status": "ok",
            "message": "tasks",
        }))
    }
    pub async fn teams() -> impl IntoResponse {
        Json(json!({
            "status": "ok",
            "message": "teams",
        }))
    }
}
