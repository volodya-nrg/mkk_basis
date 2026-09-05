use axum::{Extension, Json};
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use std::marker::PhantomData;
use uuid::Uuid;

use crate::adapter::email::EmailSender;
use crate::transport::{
    mapper,
    models::{RequestLimitOffset, RequestTaskComment, ResponseTaskCommentsList},
};
use crate::transport::models::AuthUser;
use crate::usecase::UseCase;

pub struct Handlers<ES> {
    _marker_es: PhantomData<ES>,
}

impl<ES> Handlers<ES>
where
    ES: EmailSender,
{
    pub async fn list(
        // _user: AuthenticatedUser<ES>,
        Extension(_user): Extension<AuthUser>,
        Path(task_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLimitOffset>,
    ) -> impl IntoResponse {
        use_case
            .task_comments
            .list(task_id, payload.limit, payload.offset)
            .await
            .map_or_else(
                |e| e.into_response(),
                |(items, total)| {
                    let resp = ResponseTaskCommentsList {
                        items: items
                            .into_iter()
                            .map(mapper::task_comment_uc_to_task_comment_tr)
                            .collect(),
                        total: total as u32,
                    };
                    (StatusCode::OK, Json(json!(resp))).into_response()
                },
            )
    }
    pub async fn create(
        // user: AuthenticatedUser<ES>,
        Extension(user): Extension<AuthUser>,
        Path(task_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestTaskComment>,
    ) -> impl IntoResponse {
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

        use_case.task_comments.one(new_uuid).await.map_or_else(
            |e| e.into_response(),
            |v| {
                (
                    StatusCode::OK,
                    Json(json!(mapper::task_comment_uc_to_task_comment_tr(v))),
                )
                    .into_response()
            },
        )
    }
    pub async fn delete(
        // _user: AuthenticatedUser<ES>,
        Extension(_user): Extension<AuthUser>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse {
        use_case
            .task_comments
            .delete(item_id)
            .await
            .map_or_else(|e| e.into_response(), |_| StatusCode::OK.into_response())
    }
}
