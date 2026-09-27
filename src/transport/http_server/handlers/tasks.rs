use axum::extract::Query;
use axum::response::Response;
use axum::{
    Extension, Json, extract::Path, extract::State, http::StatusCode, response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::transport::http_server::TransportState;
use crate::transport::http_server::handlers::{HandlerError, handler_err};
use crate::transport::models::{AuthUser, Task};
use crate::transport::{
    mapper,
    models::{RequestTask, RequestTaskData, ResponseMsg, TaskHistories, TasksList},
};

#[utoipa::path(
    get,
    path = "/api/v1/tasks",
    params(RequestTaskData),
    operation_id = "tasks_list",
    responses(
        (status = 200, description = "Получение списка", body = TasksList),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks",
)]
pub async fn list(
    Extension(_user): Extension<AuthUser>,
    State(state): State<Arc<TransportState>>,
    Query(payload): Query<RequestTaskData>,
) -> Response {
    let request_task_data = match mapper::task_data_tr_to_task_data_uc(payload) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ResponseMsg { msg: e.to_string() }),
            )
                .into_response();
        }
    };

    state
        .use_case
        .tasks
        .list(request_task_data)
        .await
        .map_or_else(
            |e| handler_err!(e).into_response(),
            |(items, total)| {
                Json(TasksList {
                    items: items.into_iter().map(mapper::task_uc_to_task_tr).collect(),
                    total: total as u32,
                })
                .into_response()
            },
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{id}",
    operation_id = "tasks_one",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 200, description = "Получение задачи", body = Task),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks",
)]
pub async fn one(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> Response {
    state.use_case.tasks.one(item_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| Json(mapper::task_uc_to_task_tr(v)).into_response(),
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks",
    operation_id = "tasks_create",
    request_body = RequestTask,
    responses(
        (status = 201, description = "Создание задачи", body = Task),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks",
)]
pub async fn create(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestTask>,
) -> Response {
    let uc_task = match mapper::task_tr_to_task_uc(payload) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ResponseMsg { msg: e.to_string() }),
            )
                .into_response();
        }
    };
    let new_uuid = match state.use_case.tasks.create(uc_task, user.user_id).await {
        Ok(v) => v,
        Err(e) => return handler_err!(e).into_response(),
    };

    state.use_case.tasks.one(new_uuid).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| (StatusCode::CREATED, Json(mapper::task_uc_to_task_tr(v))).into_response(),
    )
}

#[utoipa::path(
    put,
    path = "/api/v1/tasks/{id}",
    operation_id = "tasks_update",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    request_body = RequestTask,
    responses(
        (status = 200, description = "Обновление задачи", body = Task),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks",
)]
pub async fn update(
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestTask>,
) -> Response {
    let mut uc_task = match mapper::task_tr_to_task_uc(payload) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ResponseMsg { msg: e.to_string() }),
            )
                .into_response();
        }
    };

    uc_task.task_id = task_id;

    if let Err(e) = state.use_case.tasks.update(uc_task, user.user_id).await {
        return handler_err!(e).into_response();
    };

    state.use_case.tasks.one(task_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| Json(mapper::task_uc_to_task_tr(v)).into_response(),
    )
}

#[utoipa::path(
    delete,
    path = "/api/v1/tasks/{id}",
    operation_id = "tasks_delete",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 204, description = "Удаление задачи"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks",
)]
pub async fn delete(
    Extension(user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> Response {
    state
        .use_case
        .tasks
        .delete(item_id, user.user_id)
        .await
        .map_or_else(
            |e| handler_err!(e).into_response(),
            |_| StatusCode::NO_CONTENT.into_response(),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{id}/history",
    operation_id = "tasks_history",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 200, description = "Получение историй действий над задачей", body = TaskHistories),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks",
)]
pub async fn history(
    Extension(_user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> Response {
    state.use_case.tasks.get_history(task_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| {
            Json(TaskHistories {
                items: v
                    .into_iter()
                    .map(mapper::task_history_uc_to_task_history_tr)
                    .collect(),
            })
            .into_response()
        },
    )
}
