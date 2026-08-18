use crate::transport::{
    extractor::AuthenticatedUser,
    models::{RequestLogin, RequestRegister, ResponseLogin, ResponseUUID},
};
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
                let resp = ResponseUUID { uuid: new_uuid };
                (StatusCode::OK, Json(json!(resp))).into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    pub async fn login(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse {
        match use_case.auth.login(payload.email, payload.password).await {
            Ok((access_token, refresh_token)) => {
                let resp = ResponseLogin {
                    access_token: access_token.to_string(),
                    refresh_token: refresh_token.to_string(),
                };
                (StatusCode::OK, Json(resp)).into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    pub async fn logout(
        _user: AuthenticatedUser,
        State(use_case): State<UseCase>,
    ) -> impl IntoResponse {
        if let Err(e) = use_case.auth.logout().await {
            e.into_response()
        } else {
            StatusCode::OK.into_response()
        }
    }
}
