use crate::adapter::email::EmailSender;
use crate::transport::{
    extractor::AuthenticatedUser,
    mapper,
    models::{RequestLimitOffset, RequestUser, ResponseUsersList},
};
use crate::usecase::UseCase;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

pub struct Handlers {}

impl Handlers {
    pub async fn list<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLimitOffset>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        match use_case.users.list(payload.limit, payload.offset).await {
            Ok((items, total)) => {
                let resp = ResponseUsersList {
                    items: items.into_iter().map(mapper::user_uc_to_user_tr).collect(),
                    total: total as u32,
                };
                (StatusCode::OK, Json(json!(resp))).into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    pub async fn one<ES>(
        _user: AuthenticatedUser<ES>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        match use_case.users.one(item_id).await {
            Ok(v) => {
                let resp = mapper::user_uc_to_user_tr(v);
                (StatusCode::OK, Json(json!(resp))).into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    pub async fn create<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestUser>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let result = use_case
            .users
            .create(mapper::user_tr_to_user_uc(payload))
            .await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let user_uc = match use_case.users.one(new_uuid).await {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let resp = mapper::user_uc_to_user_tr(user_uc);

        (StatusCode::OK, Json(json!(resp))).into_response()
    }
    pub async fn update<ES>(
        _user: AuthenticatedUser<ES>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestUser>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let mut uc_user = mapper::user_tr_to_user_uc(payload);
        uc_user.user_id = item_id;

        if let Err(e) = use_case.users.update(uc_user).await {
            return e.into_response();
        }

        let user_uc = match use_case.users.one(item_id).await {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let resp = mapper::user_uc_to_user_tr(user_uc);

        (StatusCode::OK, Json(json!(resp))).into_response()
    }
    pub async fn delete<ES>(
        _user: AuthenticatedUser<ES>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        if let Err(e) = use_case.users.delete(item_id).await {
            e.into_response()
        } else {
            StatusCode::OK.into_response()
        }
    }
}
