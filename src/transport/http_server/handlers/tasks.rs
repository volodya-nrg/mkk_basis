use axum::extract::Query;
use axum::response::Response;
use axum::{
    Extension, Json, extract::Path, extract::State, http::StatusCode, response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::transport::http_server::handlers::helpers::ApiErrorResponse;
use crate::transport::{
    http_server::TransportState,
    http_server::handlers::helpers::map_uc_error,
    mapper,
    models::{AuthUser, RequestTask, RequestTaskData, Task, TaskHistories, TasksList},
};

#[utoipa::path(
    get,
    path = "/api/v1/tasks",
    params(RequestTaskData),
    operation_id = "tasks_list",
    responses(
        (status = 200, description = "Получение списка", body = TasksList),
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "tasks",
)]
pub async fn list(
    Extension(_user): Extension<AuthUser>,
    State(state): State<Arc<TransportState>>,
    Query(payload): Query<RequestTaskData>,
) -> axum_anyhow::ApiResult<Response> {
    let request_task_data = mapper::task_data_tr_to_task_data_uc(payload).map_err(|e| {
        axum_anyhow::ApiError::builder()
            .status(StatusCode::BAD_REQUEST)
            .title(e.to_string())
            .build()
    })?;
    let (items, total) = state
        .use_case
        .tasks
        .list(request_task_data)
        .await
        .map_err(|e| map_uc_error(e, "tasks.list"))?;

    Ok(Json(TasksList {
        items: items.into_iter().map(mapper::task_uc_to_task_tr).collect(),
        total: total as u32,
    })
    .into_response())
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
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "tasks",
)]
pub async fn one(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> axum_anyhow::ApiResult<Response> {
    let t = state
        .use_case
        .tasks
        .one(item_id)
        .await
        .map_err(|e| map_uc_error(e, "tasks.one"))?;

    Ok(Json(mapper::task_uc_to_task_tr(t)).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks",
    operation_id = "tasks_create",
    request_body = RequestTask,
    responses(
        (status = 201, description = "Создание задачи", body = Task),
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "tasks",
)]
pub async fn create(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestTask>,
) -> axum_anyhow::ApiResult<Response> {
    let uc_task = mapper::task_tr_to_task_uc(payload).map_err(|e| {
        axum_anyhow::ApiError::builder()
            .status(StatusCode::BAD_REQUEST)
            .title(e.to_string())
            .build()
    })?;
    let new_uuid = state
        .use_case
        .tasks
        .create(uc_task, user.user_id)
        .await
        .map_err(|e| map_uc_error(e, "tasks.create"))?;
    let t = state
        .use_case
        .tasks
        .one(new_uuid)
        .await
        .map_err(|e| map_uc_error(e, "tasks.create"))?;

    Ok((StatusCode::CREATED, Json(mapper::task_uc_to_task_tr(t))).into_response())
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
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "tasks",
)]
pub async fn update(
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestTask>,
) -> axum_anyhow::ApiResult<Response> {
    let mut uc_task = mapper::task_tr_to_task_uc(payload).map_err(|e| {
        axum_anyhow::ApiError::builder()
            .status(StatusCode::BAD_REQUEST)
            .title(e.to_string())
            .build()
    })?;

    uc_task.task_id = task_id;
    state
        .use_case
        .tasks
        .update(uc_task, user.user_id)
        .await
        .map_err(|e| map_uc_error(e, "tasks.update"))?;

    let t = state
        .use_case
        .tasks
        .one(task_id)
        .await
        .map_err(|e| map_uc_error(e, "tasks.update"))?;

    Ok(Json(mapper::task_uc_to_task_tr(t)).into_response())
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
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "tasks",
)]
pub async fn delete(
    Extension(user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> axum_anyhow::ApiResult<StatusCode> {
    state
        .use_case
        .tasks
        .delete(item_id, user.user_id)
        .await
        .map_err(|e| map_uc_error(e, "tasks.delete"))?;

    Ok(StatusCode::NO_CONTENT)
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
        (status = "default", description = "Ошибка API", body = ApiErrorResponse),
    ),
    tag = "tasks",
)]
pub async fn history(
    Extension(_user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> axum_anyhow::ApiResult<Response> {
    let items = state
        .use_case
        .tasks
        .get_history(task_id)
        .await
        .map_err(|e| map_uc_error(e, "tasks.history"))?;

    Ok(Json(TaskHistories {
        items: items
            .into_iter()
            .map(mapper::task_history_uc_to_task_history_tr)
            .collect(),
    })
    .into_response())
}
