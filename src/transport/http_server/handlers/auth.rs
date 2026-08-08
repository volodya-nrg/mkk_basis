use crate::transport::models::*;
use crate::usecase::UseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

pub struct Handlers {}

impl Handlers {
    pub async fn register(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestRegister>,
    ) -> impl IntoResponse {
        let result = use_case
            .auth
            .register(payload.email, payload.password, payload.password_confirm)
            .await;
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ResponseError { message: e })),
                );
            }
        };

        (StatusCode::OK, Json(json!(ResponseUUID { uuid: result })))
    }
    pub async fn login(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse {
        let result = use_case.auth.login(payload.email, payload.password).await;
        let (access_token, refresh_token) = match result {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ResponseError { message: e })),
                );
            }
        };

        (
            StatusCode::OK,
            Json(json!(ResponseLogin {
                access_token: access_token.to_string(),
                refresh_token: refresh_token.to_string(),
            })),
        )
    }
    pub async fn logout(State(use_case): State<UseCase>) -> impl IntoResponse {
        let result = use_case.auth.logout().await;
        let _ = match result {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ResponseError { message: e })),
                );
            }
        };

        (StatusCode::OK, Json(json!({})))
    }
}
