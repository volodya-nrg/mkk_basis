use crate::transport::extractor::AuthenticatedUser;
use crate::transport::models::*;
use crate::usecase::UseCase;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

pub struct Handlers {}

impl Handlers {
    pub async fn list(
        _user: AuthenticatedUser,
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestLimitOffset>,
    ) -> impl IntoResponse {
        StatusCode::OK
    }
    pub async fn get(
        _user: AuthenticatedUser,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase>,
    ) -> impl IntoResponse {
        StatusCode::OK
    }
    pub async fn create(
        _user: AuthenticatedUser,
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestUser>,
    ) -> impl IntoResponse {
        StatusCode::OK
    }
    pub async fn update(
        _user: AuthenticatedUser,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestUser>,
    ) -> impl IntoResponse {
        StatusCode::OK
    }
    pub async fn delete(
        _user: AuthenticatedUser,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase>,
    ) -> impl IntoResponse {
        StatusCode::OK
    }
}
