use axum::extract::State;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::transport::http_server::TransportState;
use crate::transport::http_server::handlers::{HandlerError, handler_err};
use crate::transport::models::{AuthUser, ResponseMsg, Team};
use crate::transport::{
    mapper,
    models::{RequestLimitOffset, RequestTeam, RequestTeamInvite, TeamsList},
};

#[utoipa::path(
    get,
    path = "/api/v1/teams",
    operation_id = "teams_list",
    params(RequestLimitOffset),
    responses(
        (status = 200, description = "Получение списка", body = TeamsList),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "teams",
)]
pub async fn list(
    Extension(_user): Extension<AuthUser>,
    State(state): State<Arc<TransportState>>,
    Query(payload): Query<RequestLimitOffset>,
) -> Response {
    state
        .use_case
        .teams
        .list(payload.limit.unwrap_or(0), payload.offset.unwrap_or(0))
        .await
        .map_or_else(
            |e| handler_err!(e).into_response(),
            |(items, total)| {
                Json(TeamsList {
                    items: items.into_iter().map(mapper::team_uc_to_team_tr).collect(),
                    total: total as u32,
                })
                .into_response()
            },
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/teams/{id}",
    operation_id = "teams_one",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 200, description = "Получение команды", body = Team),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "teams",
)]
pub async fn one(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> Response {
    state.use_case.teams.one(item_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| Json(mapper::team_uc_to_team_tr(v)).into_response(),
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/teams",
    operation_id = "teams_create",
    request_body = RequestTeam,
    responses(
        (status = 201, description = "Создание команды", body = Team),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "teams",
)]
pub async fn create(
    Extension(user): Extension<AuthUser>,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestTeam>,
) -> Response {
    let mut team_uc = mapper::team_tr_to_team_uc(payload);
    team_uc.created_by = user.user_id; // зададим id профиля

    let new_uuid = match state.use_case.teams.create(team_uc).await {
        Ok(v) => v,
        Err(e) => return handler_err!(e).into_response(),
    };

    state.use_case.teams.one(new_uuid).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| (StatusCode::CREATED, Json(mapper::team_uc_to_team_tr(v))).into_response(),
    )
}

#[utoipa::path(
    put,
    path = "/api/v1/teams/{id}",
    operation_id = "teams_update",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    request_body = RequestTeam,
    responses(
        (status = 200, description = "Изменение команды", body = Team),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "teams",
)]
pub async fn update(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestTeam>,
) -> Response {
    let mut uc_team = mapper::team_tr_to_team_uc(payload);
    uc_team.team_id = item_id;

    if let Err(e) = state.use_case.teams.update(uc_team).await {
        return handler_err!(e).into_response();
    };

    state.use_case.teams.one(item_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |v| Json(mapper::team_uc_to_team_tr(v)).into_response(),
    )
}

#[utoipa::path(
    delete,
    path = "/api/v1/teams/{id}",
    operation_id = "teams_delete",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 204, description = "Удаление команды"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "teams",
)]
pub async fn delete(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> Response {
    state.use_case.teams.delete(item_id).await.map_or_else(
        |e| handler_err!(e).into_response(),
        |_| StatusCode::NO_CONTENT.into_response(),
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/teams/{id}/invite",
    operation_id = "teams_invite",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    request_body = RequestTeamInvite,
    responses(
        (status = 200, description = "Приглашение пользователя в команду"),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "teams",
)]
pub async fn invite(
    Extension(user): Extension<AuthUser>,
    Path(team_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
    Json(payload): Json<RequestTeamInvite>,
) -> Response {
    let user_id = match Uuid::parse_str(payload.user_id.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ResponseMsg { msg: e.to_string() }),
            )
                .into_response();
        }
    };

    state
        .use_case
        .teams
        .invite(user.user_id, user.role, team_id, user_id)
        .await
        .map_or_else(
            |e| handler_err!(e).into_response(),
            |_| StatusCode::OK.into_response(),
        )
}
