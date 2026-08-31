use axum::Json;
use axum::extract::multipart::MultipartError;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;

use crate::adapter::email::EmailSender;
use crate::adapter::helpers;
use crate::err_msg::ErrMsg;
use crate::transport::{
    extractor::AuthenticatedUser,
    mapper,
    models::{
        RequestLimitOffset, RequestUserCreate, RequestUserUpdate, ResponseMsg, ResponseUsersList,
    },
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
            .users
            .list(payload.limit, payload.offset)
            .await
            .map_or_else(
                |e| e.into_response(),
                |(items, total)| {
                    let resp = ResponseUsersList {
                        items: items.into_iter().map(mapper::user_uc_to_user_tr).collect(),
                        total: total as u32,
                    };
                    (StatusCode::OK, Json(resp)).into_response()
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
        use_case.users.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(mapper::user_uc_to_user_tr(v))).into_response(),
        )
    }
    pub async fn create<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        multipart: Multipart,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let m = match multipart_to_map(multipart).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("failed to execute multipart (create user): {:?}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ResponseMsg {
                        msg: ErrMsg::NotCorrectMultipartForm.as_str(),
                    }),
                )
                    .into_response();
            }
        };
        let mut req_user = RequestUserCreate {
            email: get_string_from_map(&m, "email"),
            password: get_string_from_map(&m, "password"),
            name: get_string_option_from_map(&m, "name"),
            role: get_string_option_from_map(&m, "role"),
            avatar: None,
        };

        if let Some(avatar_bytes) = m.get("avatar").cloned() {
            match upload_file(avatar_bytes) {
                Ok(v) => req_user.avatar = Some(v),
                Err(e) => return e,
            }
        }

        let result = use_case
            .users
            .create(mapper::user_create_tr_to_user_create_uc(req_user))
            .await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        use_case.users.one(new_uuid).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(mapper::user_uc_to_user_tr(v))).into_response(),
        )
    }
    pub async fn update<ES>(
        _user: AuthenticatedUser<ES>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
        multipart: Multipart,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        let m = match multipart_to_map(multipart).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("failed to execute multipart (update user): {:?}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ResponseMsg {
                        msg: ErrMsg::NotCorrectMultipartForm.as_str(),
                    }),
                )
                    .into_response();
            }
        };
        let mut req_user = RequestUserUpdate {
            email: get_string_option_from_map(&m, "email"),
            password: get_string_option_from_map(&m, "password"),
            name: get_string_option_from_map(&m, "name"),
            role: get_string_option_from_map(&m, "role"),
            avatar: None,
            is_remove_avatar: get_string_from_map(&m, "is_remove_avatar").to_lowercase() == "true",
        };

        if let Some(avatar_bytes) = m.get("avatar").cloned() {
            match upload_file(avatar_bytes) {
                Ok(v) => req_user.avatar = Some(v),
                Err(e) => return e,
            }
        }

        let mut user_uc = mapper::user_tr_update_to_user_uc_update(req_user);
        user_uc.user_id = item_id;

        if let Err(e) = use_case.users.update(user_uc).await {
            return e.into_response();
        }

        use_case.users.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(mapper::user_uc_to_user_tr(v))).into_response(),
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
            .users
            .delete(item_id)
            .await
            .map_or_else(|e| e.into_response(), |_| StatusCode::OK.into_response())
    }
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
        .unwrap_or_else(|| "".to_string())
}

fn get_string_option_from_map(map: &HashMap<String, Vec<u8>>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
}

fn upload_file(file_data: Vec<u8>) -> Result<String, Response> {
    let image_format = image::guess_format(&file_data).map_err(|e| {
        log::error!("failed to read image format: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ResponseMsg {
                msg: "server internal error".to_string(),
            }),
        )
            .into_response()
    })?;
    let extensions = image_format.extensions_str();

    if extensions.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ResponseMsg {
                msg: "unknown image type".to_string(),
            }),
        )
            .into_response());
    }

    let new_filename = format!(
        "{}_{}.{}",
        Utc::now().timestamp(),
        helpers::rand_str_limit(5),
        extensions[0],
    );

    let filepath = format!("./web/uploaded/{}", new_filename);
    let mut file = File::create(filepath.clone()).map_err(|e| {
        log::error!("failed to create file: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ResponseMsg {
                msg: "server internal error".to_string(),
            }),
        )
            .into_response()
    })?;

    file.write_all(file_data.as_slice()).map_err(|e| {
        log::error!("failed to write file-data: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ResponseMsg {
                msg: "bad file-data".to_string(),
            }),
        )
            .into_response()
    })?;

    Ok(filepath.to_string())
}
