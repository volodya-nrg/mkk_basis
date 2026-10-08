use axum::extract::multipart::MultipartError;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapter::helpers;
use crate::app_errors::AppErr;
use crate::transport::{
    http_server::TransportState,
    mapper,
    models::{
        AuthUser, RequestLimitOffset, RequestUserCreate, RequestUserUpdate, ResponseMsg, User,
        UsersList,
    },
};

use super::helpers::map_uc_error;

struct UploadErr {
    status_code: StatusCode,
    msg: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    operation_id = "users_list",
    params(RequestLimitOffset),
    responses(
        (status = 200, description = "Получение списка", body = UsersList),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "users",
)]
pub async fn list(
    Extension(_user): Extension<AuthUser>,
    State(state): State<Arc<TransportState>>,
    Query(payload): Query<RequestLimitOffset>,
) -> axum_anyhow::ApiResult<Response> {
    let (items, total) = state
        .use_case
        .users
        .list(payload.limit.unwrap_or(0), payload.offset.unwrap_or(0))
        .await
        .map_err(|e| map_uc_error(e, "users.list"))?;

    Ok(Json(UsersList {
        items: items.into_iter().map(mapper::user_uc_to_user_tr).collect(),
        total: total as u32,
    })
    .into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    operation_id = "users_one",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 200, description = "Получение пользователя", body = User),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "users",
)]
pub async fn one(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> axum_anyhow::ApiResult<Response> {
    let user = state
        .use_case
        .users
        .one(item_id)
        .await
        .map_err(|e| map_uc_error(e, "users.one"))?;
    
    Ok(Json(mapper::user_uc_to_user_tr(user)).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    operation_id = "users_create",
    request_body = RequestUserCreate,
    responses(
        (status = 201, description = "Создание пользователя", body = User),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "users",
)]
pub async fn create(
    Extension(_user): Extension<AuthUser>,
    State(state): State<Arc<TransportState>>,
    multipart: Multipart,
) -> axum_anyhow::ApiResult<Response> {
    let m = multipart_to_map(multipart).await.map_err(|e| {
        log::error!("failed to execute multipart (create user): {:?}", e);
        axum_anyhow::ApiError::builder()
            .status(StatusCode::BAD_REQUEST)
            .title(AppErr::NotCorrectMultipartForm.to_string())
            .build()
    })?;
    let mut req_user = RequestUserCreate {
        email: get_string_from_map(&m, "email"),
        password: get_string_from_map(&m, "password"),
        name: get_string_option_from_map(&m, "name"),
        role: get_string_option_from_map(&m, "role"),
        avatar: None,
    };

    if let Some(avatar_bytes) = m.get("avatar").cloned() {
        let avatar = upload_file(avatar_bytes).map_err(|e| {
            axum_anyhow::ApiError::builder()
                .status(e.status_code)
                .title(e.msg)
                .build()
        })?;
        req_user.avatar = Some(avatar)
    }

    let new_uuid = state
        .use_case
        .users
        .create(mapper::user_create_tr_to_user_create_uc(req_user))
        .await
        .map_err(|e| map_uc_error(e, "users.create"))?;
    let user = state
        .use_case
        .users
        .one(new_uuid)
        .await
        .map_err(|e| map_uc_error(e, "users.create"))?;

    Ok((StatusCode::CREATED, Json(mapper::user_uc_to_user_tr(user))).into_response())
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/{id}",
    operation_id = "users_update",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    request_body = RequestUserUpdate,
    responses(
        (status = 200, description = "Обновление пользователя", body = User),
        (status = 400, description = "Некорректный запрос", body = ResponseMsg),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "users",
)]
pub async fn update(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
    multipart: Multipart,
) -> axum_anyhow::ApiResult<Response> {
    let m = multipart_to_map(multipart).await.map_err(|e| {
        log::error!("failed to execute multipart (update user): {:?}", e);
        axum_anyhow::ApiError::builder()
            .status(StatusCode::BAD_REQUEST)
            .title(AppErr::NotCorrectMultipartForm.to_string())
            .build()
    })?;
    let mut req_user = RequestUserUpdate {
        email: get_string_option_from_map(&m, "email"),
        password: get_string_option_from_map(&m, "password"),
        name: get_string_option_from_map(&m, "name"),
        role: get_string_option_from_map(&m, "role"),
        avatar: None,
        is_remove_avatar: get_string_from_map(&m, "is_remove_avatar").to_lowercase() == "true",
    };

    if let Some(avatar_bytes) = m.get("avatar").cloned() {
        let url = upload_file(avatar_bytes).map_err(|e| {
            axum_anyhow::ApiError::builder()
                .status(e.status_code)
                .title(e.msg)
                .build()
        })?;
        req_user.avatar = Some(url);
    }

    let mut user_uc = mapper::user_tr_update_to_user_uc_update(req_user);
    user_uc.user_id = item_id;

    state
        .use_case
        .users
        .update(user_uc)
        .await
        .map_err(|e| map_uc_error(e, "users.update"))?;

    Ok(Json(mapper::user_uc_to_user_tr(
        state
            .use_case
            .users
            .one(item_id)
            .await
            .map_err(|e| map_uc_error(e, "users.update"))?,
    ))
    .into_response())
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    operation_id = "users_delete",
    params(
        ("id" = String, Path, description = "uuid"),
    ),
    responses(
        (status = 204, description = "Удаление пользователя"),
        (status = 400, description = "Некорректный запрос"),
        (status = 500, description = "Внутренняя ошибка сервера"),
    ),
    tag = "users",
)]
pub async fn delete(
    Extension(_user): Extension<AuthUser>,
    Path(item_id): Path<Uuid>,
    State(state): State<Arc<TransportState>>,
) -> axum_anyhow::ApiResult<StatusCode> {
    state
        .use_case
        .users
        .delete(item_id)
        .await
        .map_err(|e| map_uc_error(e, "users.delete"))?;
    
    Ok(StatusCode::NO_CONTENT)
}

async fn multipart_to_map(
    mut multipart: Multipart,
) -> Result<HashMap<String, Vec<u8>>, MultipartError> {
    let mut m: HashMap<String, Vec<u8>> = HashMap::new();

    // "continue" лучше не использовать, т.к. если блок не прочитается, будет deadlock
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or_default().to_string();
        let data = field.bytes().await.unwrap_or_default();
        m.insert(name, data.to_vec());
    }

    Ok(m)
}

fn get_string_from_map(map: &HashMap<String, Vec<u8>>, key: &str) -> String {
    map.get(key)
        .and_then(|b| String::from_utf8(b.clone()).ok())
        .unwrap_or_default()
}

fn get_string_option_from_map(map: &HashMap<String, Vec<u8>>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
}

fn upload_file(file_data: Vec<u8>) -> Result<String, UploadErr> {
    let filepath = {
        let ext = image::guess_format(&file_data)
            .map_err(|e| {
                log::error!("failed to read image format: {}", e);
                UploadErr {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    msg: String::new(),
                }
            })?
            .extensions_str()
            .first()
            .ok_or_else(|| UploadErr {
                status_code: StatusCode::BAD_REQUEST,
                msg: AppErr::UndefinedTypeImage.to_string(),
            })?;
        let new_filename = format!(
            "{}_{}.{}",
            Utc::now().timestamp(),
            helpers::rand_str_limit(5),
            ext,
        );
        format!("./web/uploaded/{}", new_filename)
    };

    File::create(filepath.clone())
        .map_err(|e| {
            log::error!("failed to create file: {}", e);
            UploadErr {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: String::new(),
            }
        })?
        .write_all(file_data.as_slice())
        .map_err(|e| {
            log::error!("failed to write file-data: {}", e);
            UploadErr {
                status_code: StatusCode::BAD_REQUEST,
                msg: AppErr::BadFileData.to_string(),
            }
        })?;

    Ok(filepath)
}
