use axum::Json;
// use axum::body::Bytes;
// use axum::extract::multipart::Field;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
// use lettre::transport::smtp::client::CertificateStore::Default;
use serde_json::json;
use uuid::Uuid;

use crate::adapter::email::EmailSender;
use crate::transport::{
    extractor::AuthenticatedUser,
    mapper,
    models::{RequestLimitOffset, RequestUser, ResponseUsersList},
};
use crate::usecase::UseCase;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

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
        use_case.users.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| {
                let resp = mapper::user_uc_to_user_tr(v);
                (StatusCode::OK, Json(json!(resp))).into_response()
            },
        )
    }
    pub async fn create<ES>(
        _user: AuthenticatedUser<ES>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestUser>,
        // multipart: Multipart,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        // let (user_data, file_data) = match handle_multipart(multipart).await {
        //     Ok(v) => v,
        //     Err(e) => return e.into_response(),
        // };
        let result = use_case
            .users
            .create(mapper::user_tr_to_user_uc(payload), vec![])
            .await;
        let new_uuid = match result {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };

        use_case.users.one(new_uuid).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(json!(mapper::user_uc_to_user_tr(v)))).into_response(),
        )
    }
    pub async fn update<ES>(
        _user: AuthenticatedUser<ES>,
        Path(item_id): Path<Uuid>,
        State(use_case): State<UseCase<ES>>,
        Json(payload): Json<RequestUser>,
        // multipart: Multipart,
    ) -> impl IntoResponse
    where
        ES: EmailSender,
    {
        // let (user_data, file_data) = match handle_multipart(multipart).await {
        //     Ok(v) => v,
        //     Err(e) => return e.into_response(),
        // };

        let mut uc_user = mapper::user_tr_to_user_uc(payload);
        uc_user.user_id = item_id;

        if let Err(e) = use_case.users.update(uc_user, vec![]).await {
            return e.into_response();
        }

        use_case.users.one(item_id).await.map_or_else(
            |e| e.into_response(),
            |v| (StatusCode::OK, Json(json!(mapper::user_uc_to_user_tr(v)))).into_response(),
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

async fn handle_multipart(mut multipart: Multipart) -> Result<(RequestUser, Vec<u8>), StatusCode> {
    let mut user_data = RequestUser {
        email: "".to_string(),
        password: "".to_string(),
        name: None,
        email_code: None,
        role: None,
    };
    let mut file_data: Vec<u8> = Vec::new();

    while let field_result = multipart.next_field().await {
        let field = match field_result {
            Ok(v) => match v {
                Some(field_loc) => field_loc,
                None => continue,
            },
            Err(e) => {
                log::error!("failed to handle multipart-data: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        // берем только два поля
        match field.name() {
            Some("user_data") => {
                let bytes = field.bytes().await.unwrap_or_default();
                user_data = serde_json::from_slice(&bytes).unwrap_or_default();
            }
            Some("avatar") => {
                file_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }

    Ok((user_data, file_data))
}
