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
        match result {
            Ok(new_uuid) => {
                (StatusCode::OK, Json(json!(ResponseUUID { uuid: new_uuid }))).into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    pub async fn login(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse {
        match use_case.auth.login(payload.email, payload.password).await {
            Ok((access_token, refresh_token)) => (
                StatusCode::OK,
                Json(ResponseLogin {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                }),
            )
                .into_response(),
            Err(e) => e.into_response(),
        }
    }
    pub async fn logout(State(use_case): State<UseCase>) -> impl IntoResponse {
        match use_case.auth.logout().await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => e.into_response(),
        }
    }
}
