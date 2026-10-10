use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use std::sync::Arc;
use time::Duration;

use crate::consts;
use crate::transport::http_server::handlers::helpers::ApiErrorResponse;
use crate::transport::{
    http_server::TransportState,
    http_server::handlers::helpers::map_uc_error,
    models::{RequestLogin, RequestRegister, RequestRegisterConfirm, ResponseUuid},
};
use crate::usecase::errors::UseCaseError;

#[utoipa::path(
    post,
    path = "/api/v1/register",
    operation_id = "auth_register",
    request_body = RequestRegister,
    responses(
        (status = 200, description = "Пользователь зарегистрирован", body = ResponseUuid),
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "auth",
)]
pub async fn register(
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestRegister>,
) -> axum_anyhow::ApiResult<Response> {
    let new_uuid = state
        .use_case
        .auth
        .register(
            &payload.email,
            &payload.password,
            &payload.password_confirm,
            payload.agreement,
            payload.privacy_policy,
        )
        .await
        .map_err(|e| map_uc_error(e, "auth.register"))?;

    Ok(Json(ResponseUuid {
        value: new_uuid.to_string(),
    })
    .into_response())
}

#[utoipa::path(
    get,
    path = "/register/confirm",
    operation_id = "auth_register_confirm",
    params(RequestRegisterConfirm),
    responses(
        (status = 204, description = "Подтверждение е-мэйла на валидность и завершение регистрации"),
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "auth",
)]
pub async fn register_confirm(
    State(state): State<Arc<TransportState>>,
    Query(req): Query<RequestRegisterConfirm>,
) -> axum_anyhow::ApiResult<StatusCode> {
    state
        .use_case
        .auth
        .register_confirm(&req.email, &req.code)
        .await
        .map_err(|e| map_uc_error(e, "auth.register_confirm"))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/login",
    operation_id = "auth_login",
    request_body = RequestLogin,
    responses(
        (status = 204, description = "Аутентификация в системе"),
        (status = 303, description = "Редирект на главную страницу", body = ApiErrorResponse),
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "auth",
)]
pub async fn login(
    jar: CookieJar,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestLogin>,
) -> axum_anyhow::ApiResult<Response> {
    let login_result = state
        .use_case
        .auth
        .login(&payload.email, &payload.password)
        .await;
    let (access_token, refresh_token) = match login_result {
        Ok(tokens) => tokens,
        Err(e)
            if e.downcast_ref::<UseCaseError>()
                .is_some_and(|e| matches!(e, UseCaseError::ItemNotFound)) =>
        {
            return Ok(Redirect::to("/").into_response());
        }
        Err(e) => return Err(map_uc_error(e, "auth.login")),
    };
    let updated_jar = jar
        .add(new_cookie_for_access(
            access_token,
            consts::ACCESS_TOKEN_TTL_SEC,
        ))
        .add(new_cookie_for_refresh(
            refresh_token,
            consts::REFRESH_TOKEN_TTL_SEC,
        ));

    Ok((StatusCode::NO_CONTENT, updated_jar).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/logout",
    operation_id = "auth_logout",
    responses(
        (status = 204, description = "Выход из системы"),
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "auth",
)]
pub async fn logout(
    jar: CookieJar, // CookieJar не хранит Path и Domain, нужно указывать явно из оригинального Set-Cookie заголовка
) -> axum_anyhow::ApiResult<Response> {
    let updated_jar = jar
        .add(new_cookie_for_access(String::new(), 0))
        .add(new_cookie_for_refresh(String::new(), 0));
    Ok((StatusCode::NO_CONTENT, updated_jar).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/refresh_tokens",
    operation_id = "auth_refresh_tokens",
    responses(
        (status = 204, description = "Обновление токенов"),
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "auth",
)]
pub async fn refresh_tokens(
    jar: CookieJar,
    State(state): State<Arc<TransportState>>,
) -> axum_anyhow::ApiResult<Response> {
    let Some(cookie) = jar.get(consts::REFRESH_TOKEN_NAME) else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };
    let cookie_str = cookie.to_string();
    let Some(refresh_token_src) = cookie_str.split('=').nth(1) else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };
    let (access_token, refresh_token) = state
        .use_case
        .auth
        .refresh_tokens(refresh_token_src)
        .await
        .map_err(|e| map_uc_error(e, "auth.refresh_tokens"))?;
    let updated_jar = jar
        .add(new_cookie_for_access(
            access_token,
            consts::ACCESS_TOKEN_TTL_SEC,
        ))
        .add(new_cookie_for_refresh(
            refresh_token,
            consts::REFRESH_TOKEN_TTL_SEC,
        ));

    Ok((StatusCode::NO_CONTENT, updated_jar).into_response())
}

// В итоге ниже две ф-ии сделать так чтоб отдавали структуру Cookie. Path(Domain) при удалении надо
// чтоб совпадали, иначе куки не удалятся.
fn new_cookie_for_access(token: String, ttl: u64) -> Cookie<'static> {
    // Lax - менее строгая проверка, но хороший компрамис.
    // Кука может отправляется и с др. доменов (Telegram/почты/Google), но только для GET-запросов
    // (переходе по ссылке).
    Cookie::build((consts::ACCESS_TOKEN_NAME, token))
        .max_age(Duration::new(ttl as i64, 0))
        .same_site(SameSite::Lax)
        .secure(true)
        .http_only(true)
        .path("/api")
        .build()
}
fn new_cookie_for_refresh(token: String, ttl: u64) -> Cookie<'static> {
    // Strict - строгая проверка. Данная куки шлется только с данного домена и ни какого с другого.
    Cookie::build((consts::REFRESH_TOKEN_NAME, token))
        .max_age(Duration::new(ttl as i64, 0))
        .same_site(SameSite::Strict)
        .secure(true)
        .http_only(true)
        .path("/api/v1/refresh_tokens")
        .build()
}
