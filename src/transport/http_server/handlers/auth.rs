use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{AppendHeaders, IntoResponse, Redirect};
use axum_extra::extract::cookie::CookieJar;
use http::header;
use std::marker::PhantomData;

use crate::adapter::email::EmailSender;
use crate::consts;
use crate::transport::{
    http_server::handlers::HandlerError,
    models::{RequestLogin, RequestRegister, RequestRegisterConfirm, ResponseUuid},
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
                &payload.email,
                &payload.password,
                &payload.password_confirm,
                payload.agreement,
                payload.privacy_policy,
            )
            .await
            .map_or_else(
                |e| {
                    HandlerError {
                        source: e,
                        handler: "auth->register",
                    }
                    .into_response()
                },
                |new_uuid| {
                    Json(ResponseUuid {
                        value: new_uuid.to_string(),
                    })
                    .into_response()
                },
            )
    }
    pub async fn register_confirm(
        State(use_case): State<UseCase<ES>>,
        Query(query): Query<RequestRegisterConfirm>,
    ) -> impl IntoResponse {
        use_case
            .auth
            .register_confirm(&query.email, &query.code)
            .await
            .map_or_else(
                |e| {
                    HandlerError {
                        source: e,
                        handler: "auth->register-confirm",
                    }
                    .into_response()
                },
                |_| StatusCode::NO_CONTENT.into_response(),
            )
    }
    pub async fn login(
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse {
        let result = use_case.auth.login(&payload.email, &payload.password).await;
        let (access_token, refresh_token) = match result {
            Ok(v) => v,
            Err(e) => {
                return match e {
                    UseCaseError::UserNotExists => Redirect::to("/").into_response(),
                    _ => HandlerError {
                        source: e,
                        handler: "auth->login",
                    }
                    .into_response(),
                };
            }
        };

        (
            StatusCode::NO_CONTENT,
            AppendHeaders([
                (header::SET_COOKIE, new_cookie_for_access(access_token)),
                (header::SET_COOKIE, new_cookie_for_refresh(refresh_token)),
            ]),
        )
            .into_response()
    }
    pub async fn logout(jar: CookieJar, State(_use_case): State<UseCase<ES>>) -> impl IntoResponse {
        let jar = jar
            .remove(consts::ACCESS_TOKEN_NAME)
            .remove(consts::REFRESH_TOKEN_NAME);
        (StatusCode::NO_CONTENT, jar).into_response()
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
        let (access_token, refresh_token) =
            match use_case.auth.refresh_tokens(refresh_token_src).await {
                Ok(v) => v,
                Err(e) => {
                    return HandlerError {
                        source: e,
                        handler: "auth->refresh-tokens",
                    }
                    .into_response();
                }
            };

        (
            StatusCode::NO_CONTENT,
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
