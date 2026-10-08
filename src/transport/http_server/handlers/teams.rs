use axum::extract::State;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::transport::{
    http_server::TransportState,
    http_server::handlers::helpers::map_uc_error,
    mapper,
    models::{
        AuthUser, RequestLimitOffset, RequestTeam, RequestTeamInvite, ResponseMsg, Team, TeamsList,
    },
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
) -> axum_anyhow::ApiResult<Response> {
    let (items, total) = state
        .use_case
        .teams
        .list(payload.limit.unwrap_or(0), payload.offset.unwrap_or(0))
        .await
        .map_err(|e| map_uc_error(e, "teams.list"))?;

    Ok(Json(TeamsList {
        items: items.into_iter().map(mapper::team_uc_to_team_tr).collect(),
        total: total as u32,
    })
    .into_response())
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
) -> axum_anyhow::ApiResult<Response> {
    let t = state
        .use_case
        .teams
        .one(item_id)
        .await
        .map_err(|e| map_uc_error(e, "teams.one"))?;

    Ok(Json(mapper::team_uc_to_team_tr(t)).into_response())
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
) -> axum_anyhow::ApiResult<Response> {
    let mut team_uc = mapper::team_tr_to_team_uc(payload);
    team_uc.created_by = user.user_id; // зададим id профиля

    let new_uuid = state
        .use_case
        .teams
        .create(team_uc)
        .await
        .map_err(|e| map_uc_error(e, "teams.create"))?;
    let t = state
        .use_case
        .teams
        .one(new_uuid)
        .await
        .map_err(|e| map_uc_error(e, "teams.create"))?;

    Ok((StatusCode::CREATED, Json(mapper::team_uc_to_team_tr(t))).into_response())
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
) -> axum_anyhow::ApiResult<Response> {
    let mut uc_team = mapper::team_tr_to_team_uc(payload);
    uc_team.team_id = item_id;

    state
        .use_case
        .teams
        .update(uc_team)
        .await
        .map_err(|e| map_uc_error(e, "teams.update"))?;

    // короткая форма
    // Ok(Json(mapper::team_uc_to_team_tr(
    //     state.use_case.teams.one(item_id).await
    //         .map_err(|e| map_uc_error(e, "teams.update"))?
    // )).into_response())

    let t = state
        .use_case
        .teams
        .one(item_id)
        .await
        .map_err(|e| map_uc_error(e, "teams.update"))?;

    Ok(Json(mapper::team_uc_to_team_tr(t)).into_response())
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
) -> axum_anyhow::ApiResult<StatusCode> {
    state
        .use_case
        .teams
        .delete(item_id)
        .await
        .map_err(|e| map_uc_error(e, "teams.delete"))?;

    Ok(StatusCode::NO_CONTENT)
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
) -> axum_anyhow::ApiResult<Response> {
    let user_id = Uuid::parse_str(&payload.user_id).map_err(|e| {
        axum_anyhow::ApiError::builder()
            .status(StatusCode::BAD_REQUEST)
            .title(e.to_string())
            .build()
    })?;

    state
        .use_case
        .teams
        .invite(user.user_id, user.role, team_id, user_id)
        .await
        .map_err(|e| map_uc_error(e, "teams.invite"))?;

    Ok(StatusCode::OK.into_response())
}
