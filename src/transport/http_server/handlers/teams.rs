use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

use crate::adapter::email::EmailSender;
use crate::transport::{
    extractor::AuthenticatedUser,
    mapper,
    models::{RequestLimitOffset, RequestTeam, RequestTeamInvite, ResponseTeamsList},
};
use crate::usecase::UseCase;

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
        use_case
            .teams
            .list(payload.limit, payload.offset)
            .await
            .map_or_else(
                |e| e.into_response(),
                |(items, total)| {
                    let resp = ResponseTeamsList {
                        items: items.into_iter().map(mapper::team_uc_to_team_tr).collect(),
                        total: total as u32,
                    };
                    (StatusCode::OK, Json(json!(resp))).into_response()
                },
            )
    }
    pub async fn one<ES>(
        _user: AuthenticatedUser<ES>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        use_case.teams.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| {
                let resp = mapper::team_uc_to_team_tr(v);
                (StatusCode::OK, Json(json!(resp))).into_response()
            },
        )
    }
    pub async fn create<ES>(
        user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestTeam>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let mut team_uc = mapper::team_tr_to_team_uc(payload);
        team_uc.created_by = user.user_id; // зададим id профиля

        let new_uuid = match use_case.teams.create(team_uc).await {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        use_case.teams.one(new_uuid).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(json!(mapper::team_uc_to_team_tr(v)))).into_response(),
        )
    }
    pub async fn update<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Path(item_id): Path<Uuid>,
        Json(payload): Json<RequestTeam>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let mut uc_team = mapper::team_tr_to_team_uc(payload);
        uc_team.team_id = item_id;

        if let Err(e) = use_case.teams.update(uc_team).await {
            return e.into_response();
        };

        use_case.teams.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(json!(mapper::team_uc_to_team_tr(v)))).into_response(),
        )
    }
    pub async fn delete<ES>(
        _user: AuthenticatedUser<ES>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        use_case
            .teams
            .delete(item_id)
            .await
            .map_or_else(|e| e.into_response(), |_| StatusCode::OK.into_response())
    }
    pub async fn invite<ES>(
        user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Path(team_id): Path<Uuid>,
        Json(payload): Json<RequestTeamInvite>,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        use_case
            .teams
            .invite(user.user_id, user.role, team_id, payload.user_id)
            .await
            .map_or_else(|e| e.into_response(), |_| StatusCode::OK.into_response())
    }
}
