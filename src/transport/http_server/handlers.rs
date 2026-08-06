use crate::transport::mapper;
use crate::transport::models::*;
use crate::usecase::UseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use serde_json::json;
use uuid::Uuid;

pub struct Handlers {}

impl Handlers {
    // index
    pub async fn index() -> impl IntoResponse {
        Html(include_str!("../../../web/index.html"))
    }
    pub async fn page404() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Html(include_str!("../../../web/404.html")),
        )
            .into_response()
    }

    // auth
    pub async fn register(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestRegister>,
    ) -> impl IntoResponse {
        if payload.password != payload.password_confirm {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ResponseError {
                    message: "passwords not equal".to_string(),
                })),
            );
        }
        if !payload.is_agree {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ResponseError {
                    message: "accept agree".to_string(),
                })),
            );
        }

        let result = use_case
            .auth
            .register(payload.email, payload.password)
            .await;
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ResponseError { message: e })),
                );
            }
        };

        (StatusCode::OK, Json(json!(ResponseUUID { uuid: result })))
    }
    pub async fn login(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestLogin>,
    ) -> impl IntoResponse {
        let result = use_case.auth.login(payload.email, payload.password).await;
        let (access_token, refresh_token) = match result {
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
            Json(json!(ResponseLogin {
                access_token: access_token.to_string(),
                refresh_token: refresh_token.to_string(),
            })),
        )
    }
    pub async fn logout(State(use_case): State<UseCase>) -> impl IntoResponse {
        let result = use_case.auth.logout().await;
        let _ = match result {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!(ResponseError { message: e })),
                );
            }
        };

        (StatusCode::OK, Json(json!({})))
    }

    // teams
    pub async fn teams_list(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestLimitOffsetFilter>,
    ) -> impl IntoResponse {
        let result = use_case.teams.get_list(payload.limit, payload.offset).await;
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
            Json(json!(ResponseTeamsList {
                items: items
                    .into_iter()
                    .map(|item| mapper::team_uc_to_team_tr(item))
                    .collect(),
                total: total as u32,
            })),
        )
    }
    pub async fn teams_create(
        State(use_case): State<UseCase>,
        Json(payload): Json<RequestTeamCreate>,
    ) -> impl IntoResponse {
        let result = use_case
            .teams
            .create(mapper::team_tr_to_team_uc(payload))
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
    pub async fn teams_invite(
        State(use_case): State<UseCase>,
        Path(team_id): Path<Uuid>,
        Json(payload): Json<RequestTeamInvite>,
    ) -> impl IntoResponse {
        let result = use_case.teams.invite(team_id, payload.user_id).await;
        if let Err(e) = result {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(ResponseError { message: e })),
            );
        }

        (StatusCode::CREATED, Json(json!({})))
    }

    // tasks
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
