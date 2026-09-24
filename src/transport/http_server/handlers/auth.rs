use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{AppendHeaders, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::CookieJar;
use http::header;

use crate::adapter::email::EmailSender;
use crate::consts;
use crate::transport::{
    http_server::handlers::{HandlerError, handler_err},
    models::{RequestLogin, RequestRegister, RequestRegisterConfirmQuery, ResponseUuid},
};
use crate::usecase::{UseCase, UseCaseError};

#[utoipa::path(
    post,
    path = "/api/v1/register",
    request_body = RequestRegister,
    responses(
        (status = 200, description = "Пользователь зарегистрирован", body = ResponseUuid),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "auth"
)]
pub async fn register<ES: EmailSender>(
    State(mut use_case): State<UseCase<ES>>,
    Json(payload): Json<RequestRegister>,
) -> Response {
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
            |e| handler_err!(e).into_response(),
            |new_uuid| {
                Json(ResponseUuid {
                    value: new_uuid.to_string(),
                })
                .into_response()
            },
        )
}

#[utoipa::path(
    get,
    path = "/register/confirm",
    params(RequestRegisterConfirmQuery),
    responses(
        (status = 204, description = "Подтверждение е-мэйла на валидность и завершение регистрации"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "auth"
)]
pub async fn register_confirm<ES: EmailSender>(
    State(use_case): State<UseCase<ES>>,
    Query(req): Query<RequestRegisterConfirmQuery>,
) -> Response {
    use_case
        .auth
        .register_confirm(&req.email, &req.code)
        .await
        .map_or_else(
            |e| handler_err!(e).into_response(),
            |_| StatusCode::NO_CONTENT.into_response(),
        )
}

#[utoipa::path(
    post,
    path = "/api/v1/login",
    request_body = RequestLogin,
    responses(
        (status = 204, description = "Аутентификация в системе"),
        (status = 303, description = "Редирект на главную страницу"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "auth"
)]
pub async fn login<ES: EmailSender>(
    State(use_case): State<UseCase<ES>>,
    Json(payload): Json<RequestLogin>,
) -> impl IntoResponse {
    let result = use_case.auth.login(&payload.email, &payload.password).await;
    let (access_token, refresh_token) = match result {
        Ok(v) => v,
        Err(e) => {
            return match e {
                UseCaseError::UserNotExists => Redirect::to("/").into_response(),
                _ => handler_err!(e).into_response(),
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

#[utoipa::path(
    post,
    path = "/api/v1/logout",
    responses(
        (status = 204, description = "Выход из системы"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    security(("cookie_auth" = [])),
    tag = "auth"
)]
pub async fn logout<ES: EmailSender>(
    jar: CookieJar,
    State(_use_case): State<UseCase<ES>>,
) -> Response {
    let jar = jar
        .remove(consts::ACCESS_TOKEN_NAME)
        .remove(consts::REFRESH_TOKEN_NAME);
    (StatusCode::NO_CONTENT, jar).into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/refresh_tokens",
    responses(
        (status = 204, description = "Обновление токенов"),
        (status = 400, description = "Некорректный запрос"),
        (status = 401, description = "Не аутентифицирован"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "auth"
)]
pub async fn refresh_tokens<ES: EmailSender>(
    jar: CookieJar,
    State(use_case): State<UseCase<ES>>,
) -> Response {
    let cookie_str = match jar.get(consts::REFRESH_TOKEN_NAME) {
        Some(c) => c.to_string(),
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let refresh_token_src = match cookie_str.split("=").nth(1) {
        Some(v) => v,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let (access_token, refresh_token) = match use_case.auth.refresh_tokens(refresh_token_src).await
    {
        Ok(v) => v,
        Err(e) => return handler_err!(e).into_response(),
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
