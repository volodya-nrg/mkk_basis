use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

use crate::adapter::email::EmailSender;
use crate::transport::{
    extractor::AuthenticatedUser,
    mapper,
    models::{RequestLimitOffset, RequestTaskComment, ResponseTaskCommentsList},
};
use crate::usecase::UseCase;

pub struct Handlers {}

impl Handlers {
    pub async fn list<ES>(
        _user: AuthenticatedUser<ES>,
        Path(task_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLimitOffset>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        match use_case
            .task_comments
            .list(task_id, payload.limit, payload.offset)
            .await
        {
            Ok((items, total)) => {
                let resp = ResponseTaskCommentsList {
                    items: items
                        .into_iter()
                        .map(mapper::task_comment_uc_to_task_comment_tr)
                        .collect(),
                    total: total as u32,
                };
                (StatusCode::OK, Json(json!(resp))).into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    pub async fn create<ES>(
        user: AuthenticatedUser<ES>,
        Path(task_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestTaskComment>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let result = use_case
            .task_comments
            .create(mapper::task_comment_tr_to_task_comment_uc(
                payload.msg,
                task_id,
                user.user_id,
            ))
            .await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let task_db = match use_case.task_comments.one(new_uuid).await {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let resp = mapper::task_comment_uc_to_task_comment_tr(task_db);

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
        if let Err(e) = use_case.task_comments.delete(item_id).await {
            e.into_response()
        } else {
            StatusCode::OK.into_response()
        }
    }
}
