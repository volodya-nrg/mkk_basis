use axum::response::Response;
use axum::{
    Extension, Json, extract::Path, extract::State, http::StatusCode, response::IntoResponse,
};
use uuid::Uuid;

use crate::adapter::email::EmailSender;
use crate::transport::http_server::handlers::{HandlerError, handler_err};
use crate::transport::models::{AuthUser, Task};
use crate::transport::{
    mapper,
    models::{RequestTask, RequestTaskData, ResponseMsg, TaskHistories, TasksList},
};
use crate::usecase::UseCase;

#[utoipa::path(
    get,
    path = "/api/v1/tasks",
    request_body = RequestTaskData,
    responses(
        (status = 200, description = "Получение списка", body = TasksList),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks"
)]
pub async fn list<ES: EmailSender>(
    Extension(_user): Extension<AuthUser>,
    State(use_case): State<UseCase<ES>>,
    Json(payload): Json<RequestTaskData>,
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

    use_case.tasks.list(request_task_data).await.map_or_else(
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
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 200, description = "Получение задачи", body = Task),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks"
)]
pub async fn one<ES: EmailSender>(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(use_case): State<UseCase<ES>>,
) -> Response {
    use_case.tasks.one(item_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| Json(mapper::task_uc_to_task_tr(v)).into_response(),
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks",
    request_body = RequestTask,
    responses(
        (status = 201, description = "Создание задачи", body = Task),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks"
)]
pub async fn create<ES: EmailSender>(
    Extension(user): Extension<AuthUser>,
    State(use_case): State<UseCase<ES>>,
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
    let new_uuid = match use_case.tasks.create(uc_task, user.user_id).await {
        Ok(v) => v,
        Err(e) => return handler_err!(e).into_response(),
    };

    use_case.tasks.one(new_uuid).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| (StatusCode::CREATED, Json(mapper::task_uc_to_task_tr(v))).into_response(),
    )
}

#[utoipa::path(
    put,
    path = "/api/v1/tasks/{id}",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    request_body = RequestTask,
    responses(
        (status = 200, description = "Обновление задачи", body = Task),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks"
)]
pub async fn update<ES: EmailSender>(
    Extension(user): Extension<AuthUser>,
    State(use_case): State<UseCase<ES>>,
    Path(task_id): Path<Uuid>,
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

    if let Err(e) = use_case.tasks.update(uc_task, user.user_id).await {
        return handler_err!(e).into_response();
    };

    use_case.tasks.one(task_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| Json(mapper::task_uc_to_task_tr(v)).into_response(),
    )
}

#[utoipa::path(
    delete,
    path = "/api/v1/tasks/{id}",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 204, description = "Удаление задачи"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks"
)]
pub async fn delete<ES: EmailSender>(
    Extension(user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(use_case): State<UseCase<ES>>,
) -> Response {
    use_case
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
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 200, description = "Получение историй действий над задачей", body = TaskHistories),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "tasks"
)]
pub async fn history<ES: EmailSender>(
    Extension(_user): Extension<AuthUser>,
    State(use_case): State<UseCase<ES>>,
    Path(task_id): Path<Uuid>,
) -> Response {
    use_case.tasks.get_history(task_id).await.map_or_else(
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
