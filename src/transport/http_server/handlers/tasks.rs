use crate::adapter::email::EmailSender;
use crate::transport::{
    extractor::AuthenticatedUser,
    mapper,
    models::{RequestTask, RequestTaskData, ResponseTaskHistories, ResponseTasksList},
};
use crate::usecase::UseCase;
use axum::{Json, extract::Path, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::marker::PhantomData;
use uuid::Uuid;

pub struct Handlers<ES> {
    _marker_es: PhantomData<ES>,
}

impl<ES> Handlers<ES>
where
    ES: EmailSender,
{
    pub async fn list(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestTaskData>,
    ) -> impl IntoResponse {
        use_case
            .tasks
            .list(mapper::task_data_tr_to_task_data_uc(payload))
            .await
            .map_or_else(
                |e| e.into_response(),
                |(items, total)| {
                    let resp = ResponseTasksList {
                        items: items.into_iter().map(mapper::task_uc_to_task_tr).collect(),
                        total: total as u32,
                    };
                    (StatusCode::OK, Json(json!(resp))).into_response()
                },
            )
    }
    pub async fn one(
        _user: AuthenticatedUser<ES>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse {
        use_case.tasks.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(json!(mapper::task_uc_to_task_tr(v)))).into_response(),
        )
    }
    pub async fn create(
        user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestTask>,
    ) -> impl IntoResponse {
        let result = use_case
            .tasks
            .create(mapper::task_tr_to_task_uc(payload), user.user_id)
            .await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        use_case.tasks.one(new_uuid).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(json!(mapper::task_uc_to_task_tr(v)))).into_response(),
        )
    }
    pub async fn update(
        user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Path(task_id): Path<Uuid>,
        Json(payload): Json<RequestTask>,
    ) -> impl IntoResponse {
        let mut uc_task = mapper::task_tr_to_task_uc(payload);
        uc_task.task_id = task_id;

        if let Err(e) = use_case.tasks.update(uc_task, user.user_id).await {
            return e.into_response();
        };

        use_case.tasks.one(task_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(json!(mapper::task_uc_to_task_tr(v)))).into_response(),
        )
    }
    pub async fn delete(
        user: AuthenticatedUser<ES>,
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
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Path(task_id): Path<Uuid>,
    ) -> impl IntoResponse {
        use_case.tasks.get_history(task_id).await.map_or_else(
            |e| e.into_response(),
            |v| {
                let resp = ResponseTaskHistories {
                    items: v
                        .into_iter()
                        .map(mapper::task_history_uc_to_task_history_tr)
                        .collect(),
                };
                (StatusCode::OK, Json(json!(resp))).into_response()
            },
        )
    }
}
