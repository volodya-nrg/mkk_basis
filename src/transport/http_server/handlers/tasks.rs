use crate::transport::extractor::AuthenticatedUser;
use crate::transport::mapper;
use crate::transport::models::*;
use crate::usecase::UseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

pub struct Handlers {}

impl Handlers {
    pub async fn tasks_list(
        _user: AuthenticatedUser,
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestLimitOffset>,
    ) -> impl IntoResponse {
        match use_case.tasks.get_list(payload.limit, payload.offset).await {
            Ok((items, total)) => (
                StatusCode::OK,
                Json(json!(ResponseTasksList {
                    items: items.into_iter().map(mapper::task_uc_to_task_tr).collect(),
                    total: total as u32,
                })),
            )
                .into_response(),
            Err(e) => e.into_response(),
        }
    }
    pub async fn tasks_create(
        _user: AuthenticatedUser,
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestTask>,
    ) -> impl IntoResponse {
        let result = use_case
            .tasks
            .create(mapper::task_tr_to_task_uc(payload))
            .await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        // получим запись
        let task_db = match use_case.tasks.one(new_uuid).await {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        (
            StatusCode::OK,
            Json(json!(mapper::task_uc_to_task_tr(task_db))),
        )
            .into_response()
    }

    pub async fn tasks_update(
        _user: AuthenticatedUser,
        State(use_case): State<UseCase>,
        Path(task_id): Path<Uuid>,
        Json(payload): Json<RequestTask>,
    ) -> impl IntoResponse {
        let mut uc_task = mapper::task_tr_to_task_uc(payload);
        uc_task.task_id = task_id;

        let result = use_case.tasks.update(uc_task).await;
        match result {
            Ok(_) => {}
            Err(e) => return e.into_response(),
        };

        // получим запись
        let task_db = match use_case.tasks.one(task_id).await {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        (
            StatusCode::OK,
            Json(json!(mapper::task_uc_to_task_tr(task_db))),
        )
            .into_response()
    }
    pub async fn tasks_history(
        _user: AuthenticatedUser,
        State(use_case): State<UseCase>,
        Path(task_id): Path<Uuid>,
    ) -> impl IntoResponse {
        match use_case.tasks.get_history(task_id).await {
            Ok(v) => (
                StatusCode::OK,
                Json(json!(ResponseTaskHistories {
                    items: v
                        .into_iter()
                        .map(mapper::task_history_uc_to_task_history_tr)
                        .collect(),
                })),
            )
                .into_response(),
            Err(e) => e.into_response(),
        }
    }
}
