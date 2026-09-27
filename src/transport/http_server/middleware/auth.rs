use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use http::StatusCode;
use std::sync::Arc;

use crate::adapter::jwt;
use crate::consts;
use crate::transport::http_server::TransportState;
use crate::transport::models::AuthUser;

pub async fn auth(
    jar: CookieJar,
    State(state): State<Arc<TransportState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let cook_str = jar
        .get(consts::ACCESS_TOKEN_NAME)
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();
    let token = cook_str.split("=").nth(1).ok_or(StatusCode::UNAUTHORIZED)?;
    let claim = state
        .use_case
        .auth
        .jwt_service
        .validate_access_token(token)
        .map_err(|e| {
            log::error!("{e}");
            StatusCode::UNAUTHORIZED
        })?;

    if claim.token_type != jwt::TYPE_ACCESS {
        return Err(StatusCode::UNAUTHORIZED);
    }

    req.extensions_mut().insert(AuthUser {
        user_id: claim.sub,
        role: claim.role,
    });

    Ok(next.run(req).await)
}
