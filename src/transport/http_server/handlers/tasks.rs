use axum::{
    Extension, Json, extract::Path, extract::State, http::StatusCode, response::IntoResponse,
};
use std::marker::PhantomData;
use uuid::Uuid;

use crate::adapter::email::EmailSender;
use crate::transport::models::AuthUser;
use crate::transport::{
    mapper,
    models::{RequestTask, RequestTaskData, ResponseMsg, TaskHistories, TasksList},
};
use crate::usecase::UseCase;

pub struct Handlers<ES> {
    _marker_es: PhantomData<ES>,
}

impl<ES> Handlers<ES>
where
    ES: EmailSender,
{
    pub async fn list(
        Extension(_user): Extension<AuthUser>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestTaskData>,
    ) -> impl IntoResponse {
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
            |e| e.into_response(),
            |(items, total)| {
                let resp = TasksList {
                    items: items.into_iter().map(mapper::task_uc_to_task_tr).collect(),
                    total: total as u32,
                };
                (StatusCode::OK, Json(resp)).into_response()
            },
        )
    }
    pub async fn one(
        Extension(_user): Extension<AuthUser>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse {
        use_case.tasks.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(mapper::task_uc_to_task_tr(v))).into_response(),
        )
    }
    pub async fn create(
        Extension(user): Extension<AuthUser>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestTask>,
    ) -> impl IntoResponse {
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
        let result = use_case.tasks.create(uc_task, user.user_id).await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        use_case.tasks.one(new_uuid).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(mapper::task_uc_to_task_tr(v))).into_response(),
        )
    }
    pub async fn update(
        Extension(user): Extension<AuthUser>,
        State(use_case): State<UseCase<ES>>,
        Path(task_id): Path<Uuid>,
        Json(payload): Json<RequestTask>,
    ) -> impl IntoResponse {
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
            return e.into_response();
        };

        use_case.tasks.one(task_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(mapper::task_uc_to_task_tr(v))).into_response(),
        )
    }
    pub async fn delete(
        Extension(user): Extension<AuthUser>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse {
        use_case
            .tasks
            .delete(item_id, user.user_id)
            .await
            .map_or_else(|e| e.into_response(), |_| StatusCode::OK.into_response())
    }
    pub async fn history(
        Extension(_user): Extension<AuthUser>,
        State(use_case): State<UseCase<ES>>,
        Path(task_id): Path<Uuid>,
    ) -> impl IntoResponse {
        use_case.tasks.get_history(task_id).await.map_or_else(
            |e| e.into_response(),
            |v| {
                let resp = TaskHistories {
                    items: v
                        .into_iter()
                        .map(mapper::task_history_uc_to_task_history_tr)
                        .collect(),
                };
                (StatusCode::OK, Json(resp)).into_response()
            },
        )
    }
}
