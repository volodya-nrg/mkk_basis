use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::adapter::email::EmailSender;
use crate::transport::{
    extractor::AuthenticatedUser,
    models::{
        RequestLogin, RequestRefreshToken, RequestRegister, RequestRegisterConfirm, ResponseLogin,
        ResponseRefreshToken, ResponseUUID,
    },
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
        use_case
            .auth
            .register(
                payload.email,
                payload.password,
                payload.password_confirm,
                payload.agreement,
                payload.privacy_policy,
            )
            .await
            .map_or_else(
                |e| e.into_response(),
                |new_uuid| {
                    (StatusCode::OK, Json(json!(ResponseUUID { uuid: new_uuid }))).into_response()
                },
            )
    }
    pub async fn register_confirm<ES>(
        State(use_case): State<UseCase<ES>>,
        Query(query): Query<RequestRegisterConfirm>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        use_case
            .auth
            .register_confirm(
                query.email.unwrap_or("".to_string()),
                query.code.unwrap_or("".to_string()),
            )
            .await
            .map_or_else(|e| e.into_response(), |_| StatusCode::OK.into_response())
    }
    pub async fn login<ES>(
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        use_case
            .auth
            .login(payload.email, payload.password)
            .await
            .map_or_else(
                |e| e.into_response(),
                |(access_token, refresh_token)| {
                    let resp = ResponseLogin {
                        access_token: access_token.to_string(),
                        refresh_token: refresh_token.to_string(),
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                },
            )
    }
    pub async fn logout<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        use_case
            .auth
            .logout()
            .await
            .map_or_else(|e| e.into_response(), |_| StatusCode::OK.into_response())
    }
    pub async fn refresh_tokens<ES>(
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestRefreshToken>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        use_case
            .auth
            .refresh_tokens(payload.token.to_string())
            .await
            .map_or_else(
                |e| e.into_response(),
                |(access_token, refresh_token)| {
                    let resp = ResponseRefreshToken {
                        access_token: access_token.to_string(),
                        refresh_token: refresh_token.to_string(),
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                },
            )
    }
}
