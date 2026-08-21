use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::adapter::email::EmailSender;
use crate::transport::{
    extractor::AuthenticatedUser,
    models::{RequestLogin, RequestRegister, RequestRegisterConfirm, ResponseLogin, ResponseUUID},
};
use crate::usecase::UseCase;

pub struct Handlers {}

impl Handlers {
    pub async fn register<ES>(
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestRegister>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let result = use_case
            .auth
            .register(
                payload.email,
                payload.password,
                payload.password_confirm,
                payload.agreement,
                payload.privacy_policy,
            )
            .await;
        match result {
            Ok(new_uuid) => {
                let resp = ResponseUUID { uuid: new_uuid };
                (StatusCode::OK, Json(json!(resp))).into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    pub async fn register_confirm<ES>(
        State(use_case): State<UseCase<ES>>,
        Query(query): Query<RequestRegisterConfirm>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let email = query.email.unwrap_or("".to_string());
        let code = query.code.unwrap_or("".to_string());
        match use_case.auth.register_confirm(email, code).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => e.into_response(),
        }
    }
    pub async fn login<ES>(
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
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
    pub async fn logout<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        if let Err(e) = use_case.auth.logout().await {
            e.into_response()
        } else {
            StatusCode::OK.into_response()
        }
    }
}
