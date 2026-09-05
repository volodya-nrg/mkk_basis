use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use http::StatusCode;

use crate::adapter::email::EmailSender;
use crate::adapter::jwt;
use crate::consts;
use crate::transport::models::AuthUser;
use crate::usecase::UseCase;

pub async fn auth<ES>(
    jar: CookieJar,
    State(use_case): State<UseCase<ES>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode>
where
    ES: EmailSender,
{
    let cook_str = jar
        .get(consts::ACCESS_TOKEN_NAME)
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();
    let token = cook_str.split("=").nth(1).ok_or(StatusCode::UNAUTHORIZED)?;
    let claim = use_case
        .auth
        .jwt_service
        .validate_access_token(token.to_string())
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
