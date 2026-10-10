use http::StatusCode;
use serde::Serialize;
#[allow(unused_imports)]
use serde_json::json;
use utoipa::{ToResponse, ToSchema};

use crate::usecase::errors::UseCaseError;

// так как axum_anyhow::ApiError не реализует трейт ToSchema из utoipa, то создадим свой
#[derive(ToResponse, Serialize, ToSchema)]
#[response(
    description = "Стандартная структура ошибки",
    content_type = "application/json",
    example = json!({
        "status": 400,
        "title": "Bad Request",
        "detail": "The requested resource is invalid"
    })
)]
pub struct ApiErrorResponse {
    status: u16,
    title: String,
    detail: String,
}

pub fn map_uc_error(e: anyhow::Error, handler: &str) -> axum_anyhow::ApiError {
    let mut status_code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut public_err = String::from("Internal error");
    let all: Vec<String> = e.chain().map(ToString::to_string).collect();
    let mut internal_err = all.join("; ");

    if let Some(uc) = e.chain().find_map(|c| c.downcast_ref::<UseCaseError>()) {
        match uc {
            UseCaseError::Transport {
                status_code: sc,
                public_err: pe,
                internal_err: ie,
            } => {
                status_code = *sc;
                public_err = pe.clone();

                if let Some(v) = ie {
                    internal_err = v.clone();
                }
            }
            UseCaseError::ItemNotFound => {
                status_code = StatusCode::NOT_FOUND;
                public_err = "item not found".into();
            }
            UseCaseError::Internal(e2) => {
                internal_err = format!("{e2}");
            }
        }
    }

    // тут надо логировать только 500-ые ошибки
    if status_code.is_server_error() {
        log::error!("{}; {}", handler, internal_err);
    }

    axum_anyhow::ApiError::builder()
        .status(status_code)
        .title(status_code.canonical_reason().unwrap_or("Error"))
        .detail(public_err)
        .build()
}
