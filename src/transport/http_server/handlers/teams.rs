use crate::adapter::email::EmailSender;
use crate::transport::{
    extractor::AuthenticatedUser,
    mapper,
    models::{RequestLimitOffset, RequestTeamCreate, RequestTeamInvite, ResponseTeamsList},
};
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
    pub async fn list<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLimitOffset>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        match use_case.teams.list(payload.limit, payload.offset).await {
            Ok((items, total)) => {
                let resp = ResponseTeamsList {
                    items: items.into_iter().map(mapper::team_uc_to_team_tr).collect(),
                    total: total as u32,
                };
                (StatusCode::OK, Json(json!(resp))).into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    pub async fn create<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestTeamCreate>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let result = use_case
            .teams
            .create(mapper::team_tr_to_team_uc(payload))
            .await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let team_uc = match use_case.teams.one(new_uuid).await {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let resp = mapper::team_uc_to_team_tr(team_uc);

        (StatusCode::OK, Json(json!(resp))).into_response()
    }
    pub async fn invite<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Path(team_id): Path<Uuid>,
        Json(payload): Json<RequestTeamInvite>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        if let Err(e) = use_case.teams.invite(team_id, payload.user_id).await {
            e.into_response()
        } else {
            StatusCode::OK.into_response()
        }
    }
}
