use axum::{Extension, Json};
use axum::extract::multipart::MultipartError;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::marker::PhantomData;
use uuid::Uuid;

use crate::adapter::{email::EmailSender, helpers};
use crate::err_msg::ErrMsg;
use crate::transport::{
    mapper,
    models::{
        RequestLimitOffset, RequestUserCreate, RequestUserUpdate, ResponseMsg, ResponseUsersList, AuthUser,
    },
};
use crate::usecase::UseCase;

struct UploadErr {
    status_code: StatusCode,
    msg: String,
}

pub struct Handlers<ES> {
    _marker_es: PhantomData<ES>,
}

impl<ES> Handlers<ES>
where
    ES: EmailSender,
{
    pub async fn list(
        // _user: AuthenticatedUser<ES>,
        Extension(_user): Extension<AuthUser>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestLimitOffset>,
    ) -> impl IntoResponse {
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
    pub async fn one(
        //_user: AuthenticatedUser<ES>,
        Extension(_user): Extension<AuthUser>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse {
        use_case.users.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(mapper::user_uc_to_user_tr(v))).into_response(),
        )
    }
    pub async fn create(
        //_user: AuthenticatedUser<ES>,
        Extension(_user): Extension<AuthUser>,
        State(use_case): State<UseCase<ES>>,
        multipart: Multipart,
    ) -> impl IntoResponse {
        let m = match multipart_to_map(multipart).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("failed to execute multipart (create user): {:?}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ResponseMsg {
                        msg: ErrMsg::NotCorrectMultipartForm.to_string(),
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
                Err(e) => return (e.status_code, Json(ResponseMsg { msg: e.msg })).into_response(),
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
    pub async fn update(
        //_user: AuthenticatedUser<ES>,
        Extension(_user): Extension<AuthUser>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
        multipart: Multipart,
    ) -> impl IntoResponse {
        let m = match multipart_to_map(multipart).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("failed to execute multipart (update user): {:?}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ResponseMsg {
                        msg: ErrMsg::NotCorrectMultipartForm.to_string(),
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
                Err(e) => return (e.status_code, Json(ResponseMsg { msg: e.msg })).into_response(),
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
    pub async fn delete(
        //_user: AuthenticatedUser<ES>,
        Extension(_user): Extension<AuthUser>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
    ) -> impl IntoResponse {
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
        .unwrap_or_default()
}

fn get_string_option_from_map(map: &HashMap<String, Vec<u8>>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
}

fn upload_file(file_data: Vec<u8>) -> Result<String, UploadErr> {
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
            msg: ErrMsg::UndefinedTypeImage.to_string(),
        })?;
    let new_filename = format!(
        "{}_{}.{}",
        Utc::now().timestamp(),
        helpers::rand_str_limit(5),
        ext,
    );
    let filepath = format!("./web/uploaded/{}", new_filename);

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
                msg: ErrMsg::BadFileData.to_string(),
            }
        })?;

    Ok(filepath.to_string())
}
