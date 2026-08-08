use crate::transport::mapper;
use crate::transport::models::*;
use crate::usecase::UseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse};
use serde_json::json;
use uuid::Uuid;

pub struct Handlers {}

impl Handlers {
    pub async fn tasks_list(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestLimitOffsetFilter>,
    ) -> impl IntoResponse {
        let result = use_case.tasks.get_list(payload.limit, payload.offset).await;
        let (items, total) = match result {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ResponseError { message: e })),
                );
            }
        };

        (
            StatusCode::OK,
            Json(json!(ResponseTasksList {
                items: items
                    .into_iter()
                    .map(|item| mapper::task_uc_to_task_tr(item))
                    .collect(),
                total: total as u32,
            })),
        )
    }
    pub async fn tasks_create(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestTask>,
    ) -> impl IntoResponse {
        let result = use_case
            .tasks
            .create(mapper::task_tr_to_task_uc(payload))
            .await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ResponseError { message: e })),
                );
            }
        };

        (StatusCode::OK, Json(json!(ResponseUUID { uuid: new_uuid })))
    }
    pub async fn tasks_update(
        State(use_case): State<UseCase>,
        Path(task_id): Path<Uuid>,
        Json(payload): Json<RequestTask>,
    ) -> impl IntoResponse {
        let mut uc_task = mapper::task_tr_to_task_uc(payload);
        uc_task.task_id = task_id;

        if let Err(e) = use_case.tasks.update(uc_task).await {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ResponseError { message: e })),
            );
        }

        (StatusCode::OK, Json(json!({})))
    }
    pub async fn tasks_history(
        State(use_case): State<UseCase>,
        Path(task_id): Path<Uuid>,
    ) -> impl IntoResponse {
        let result = use_case.tasks.get_history(task_id).await;
        let items = match result {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ResponseError { message: e })),
                );
            }
        };

        (
            StatusCode::OK,
            Json(json!(ResponseTaskHistories {
                items: items
                    .into_iter()
                    .map(|item| mapper::task_history_uc_to_task_history_tr(item))
                    .collect(),
            })),
        )
    }
}
