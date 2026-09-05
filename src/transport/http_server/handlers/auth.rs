use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{AppendHeaders, IntoResponse, Redirect};
use axum::{Extension, Json};
use axum_extra::extract::cookie::CookieJar;
use http::header;
use serde_json::json;
use std::marker::PhantomData;

use crate::adapter::email::EmailSender;
use crate::consts;
use crate::transport::models::AuthUser;
use crate::transport::models::{
    RequestLogin, RequestRegister, RequestRegisterConfirm, ResponseUUID,
};
use crate::usecase::{UseCase, UseCaseError};

pub struct Handlers<ES> {
    _marker_es: PhantomData<ES>,
}

impl<ES> Handlers<ES>
where
    ES: EmailSender,
{
    pub async fn register(
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestRegister>,
    ) -> impl IntoResponse {
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
    pub async fn register_confirm(
        State(use_case): State<UseCase<ES>>,
        Query(query): Query<RequestRegisterConfirm>,
    ) -> impl IntoResponse {
        use_case
            .auth
            .register_confirm(
                query.email.unwrap_or_default(),
                query.code.unwrap_or_default(),
            )
            .await
            .map_or_else(|e| e.into_response(), |_| StatusCode::OK.into_response())
    }
    pub async fn login(
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse {
        let result = use_case.auth.login(payload.email, payload.password).await;
        let (access_token, refresh_token) = match result {
            Ok(v) => v,
            Err(e) => {
                return match e {
                    UseCaseError::UserNotExists => Redirect::to("/").into_response(),
                    _ => e.into_response(),
                };
            }
        };

        (
            StatusCode::OK,
            AppendHeaders([
                (header::SET_COOKIE, new_cookie_for_access(access_token)),
                (header::SET_COOKIE, new_cookie_for_refresh(refresh_token)),
            ]),
        )
            .into_response()
    }
    pub async fn logout(
        jar: CookieJar,
        Extension(_user): Extension<AuthUser>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse {
        if let Err(e) = use_case.auth.logout().await {
            return e.into_response();
        };
        let jar = jar
            .remove(consts::ACCESS_TOKEN_NAME)
            .remove(consts::REFRESH_TOKEN_NAME);
        jar.into_response()
    }
    pub async fn refresh_tokens(
        jar: CookieJar,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse {
        let cookie_str = match jar.get(consts::REFRESH_TOKEN_NAME) {
            Some(c) => c.to_string(),
            None => return StatusCode::UNAUTHORIZED.into_response(),
        };
        let refresh_token_src = match cookie_str.split("=").nth(1) {
            Some(v) => v,
            None => return StatusCode::UNAUTHORIZED.into_response(),
        };
        let (access_token, refresh_token) = match use_case
            .auth
            .refresh_tokens(refresh_token_src.to_string())
            .await
        {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        (
            StatusCode::OK,
            AppendHeaders([
                (header::SET_COOKIE, new_cookie_for_access(access_token)),
                (header::SET_COOKIE, new_cookie_for_refresh(refresh_token)),
            ]),
        )
            .into_response()
    }
}

fn new_cookie_for_access(token: String) -> String {
    // Lax - менее строгая проверка, но хороший компрамис.
    // Кука может отправляется и с др. доменов (Telegram/почты/Google), но только для GET-запросов
    // (переходе по ссылке).
    format!(
        "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/api; Max-Age={}",
        consts::ACCESS_TOKEN_NAME,
        token,
        consts::ACCESS_TOKEN_TTL_SEC,
    )
}
fn new_cookie_for_refresh(token: String) -> String {
    // Strict - строгая проверка. Данная куки шлется только с данного домена и ни какого с другого.
    format!(
        "{}={}; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/refresh_tokens; Max-Age={}",
        consts::REFRESH_TOKEN_NAME,
        token,
        consts::REFRESH_TOKEN_TTL_SEC,
    )
}
