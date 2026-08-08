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
}
