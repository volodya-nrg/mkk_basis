use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use uuid::Uuid;

use crate::adapter::email::EmailSender;
use crate::transport::http_server::handlers::{HandlerError, handler_err};
use crate::transport::models::{AuthUser, TaskComment};
use crate::transport::{
    mapper,
    models::{RequestLimitOffset, RequestTaskComment, TaskCommentsList},
};
use crate::usecase::UseCase;

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{id}/comments",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    request_body = RequestLimitOffset,
    responses(
        (status = 200, description = "Получение списка", body = TaskCommentsList),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "task_comments"
)]
pub async fn list<ES: EmailSender>(
    // _user: AuthenticatedUser<ES>,
    Extension(_user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    State(use_case): State<UseCase<ES>>,
    Json(payload): Json<RequestLimitOffset>,
) -> Response {
    use_case
        .task_comments
        .list(task_id, payload.limit, payload.offset)
        .await
        .map_or_else(
            |e| handler_err!(e).into_response(),
            |(items, total)| {
                Json(TaskCommentsList {
                    items: items
                        .into_iter() // перебор по значениям
                        .map(mapper::task_comment_uc_to_task_comment_tr)
                        .collect(),
                    total: total as u32,
                })
                .into_response()
            },
        )
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/{id}/comments",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    request_body = RequestTaskComment,
    responses(
        (status = 201, description = "Комментарий к задаче создан", body = TaskComment),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "task_comments"
)]
pub async fn create<ES: EmailSender>(
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    State(use_case): State<UseCase<ES>>,
    Json(payload): Json<RequestTaskComment>,
) -> Response {
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
        Err(e) => return handler_err!(e).into_response(),
    };

    use_case.task_comments.one(new_uuid).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| {
            (
                StatusCode::CREATED,
                Json(mapper::task_comment_uc_to_task_comment_tr(v)),
            )
                .into_response()
        },
    )
}

#[utoipa::path(
    delete,
    path = "/api/v1/tasks/comment/{id}",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 204, description = "Комментарий к задаче удален"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "task_comments"
)]
pub async fn delete<ES: EmailSender>(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(use_case): State<UseCase<ES>>,
) -> Response {
    use_case.task_comments.delete(item_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |_| StatusCode::NO_CONTENT.into_response(),
    )
}
