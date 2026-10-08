use axum::extract::State;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::transport::{
    http_server::TransportState,
    http_server::handlers::helpers::map_uc_error,
    mapper,
    models::{AuthUser, RequestLimitOffset, RequestTaskComment, TaskComment, TaskCommentsList},
};

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{id}/comments",
    operation_id = "task_comments_list",
    params(
        ("id" = String, Path, description = "uuid"),
        RequestLimitOffset,
    ),
    responses(
        (status = 200, description = "Получение списка", body = TaskCommentsList),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "task_comments",
)]
pub async fn list(
    // _user: AuthenticatedUser<ES>,
    Extension(_user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
    Query(payload): Query<RequestLimitOffset>,
) -> axum_anyhow::ApiResult<Response> {
    let (items, total) = state
        .use_case
        .task_comments
        .list(
            task_id,
            payload.limit.unwrap_or(0),
            payload.offset.unwrap_or(0),
        )
        .await
        .map_err(|e| map_uc_error(e, "task_comments.list"))?;

    Ok(Json(TaskCommentsList {
        items: items
            .into_iter() // перебор по значениям
            .map(mapper::task_comment_uc_to_task_comment_tr)
            .collect(),
        total: total as u32,
    })
    .into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/{id}/comments",
    operation_id = "task_comments_create",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    request_body = RequestTaskComment,
    responses(
        (status = 201, description = "Комментарий к задаче создан", body = TaskComment),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "task_comments",
)]
pub async fn create(
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestTaskComment>,
) -> axum_anyhow::ApiResult<Response> {
    let new_uuid = state
        .use_case
        .task_comments
        .create(mapper::task_comment_tr_to_task_comment_uc(
            payload.msg,
            task_id,
            user.user_id,
        ))
        .await
        .map_err(|e| map_uc_error(e, "task_comments.create"))?;
    let t = state
        .use_case
        .task_comments
        .one(new_uuid)
        .await
        .map_err(|e| map_uc_error(e, "task_comments.create"))?;

    Ok((
        StatusCode::CREATED,
        Json(mapper::task_comment_uc_to_task_comment_tr(t)),
    )
        .into_response())
}

#[utoipa::path(
    delete,
    path = "/api/v1/tasks/comment/{id}",
    operation_id = "task_comments_delete",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 204, description = "Комментарий к задаче удален"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "task_comments",
)]
pub async fn delete(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> axum_anyhow::ApiResult<StatusCode> {
    state
        .use_case
        .task_comments
        .delete(item_id)
        .await
        .map_err(|e| map_uc_error(e, "task_comments.delete"))?;

    Ok(StatusCode::NO_CONTENT)
}
