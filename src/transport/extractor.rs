use crate::adapter::email::EmailSender;
use crate::usecase::UseCase;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use http::StatusCode;
use uuid::Uuid;
use std::marker::PhantomData;

pub struct AuthenticatedUser<ES> {
    #[allow(dead_code)]
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub role: String,
    pub _marker: PhantomData<ES>,
}

impl<S,ES> FromRequestParts<S> for AuthenticatedUser<ES>
where
    S: Send + Sync,
    ES: EmailSender + Clone + Send + Sync,
    UseCase<ES>: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_string();
        let use_case: UseCase<ES> = UseCase::from_ref(state);
        let claim = use_case
            .auth
            .jwt_service
            .validate_access_token(token)
            .map_err(|e| {
                log::error!("{e}");
                StatusCode::UNAUTHORIZED
            })?;

        Ok(AuthenticatedUser {
            user_id: claim.sub,
            role: claim.role,
            _marker: Default::default(),
        })  
    }
}
