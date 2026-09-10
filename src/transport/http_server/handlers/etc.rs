use axum::Json;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

use crate::transport::models::ResponseMsg;

pub struct Handlers {}

impl Handlers {
    pub async fn index() -> impl IntoResponse {
        Html(include_str!("../../../../web/index.html"))
    }
    pub async fn health() -> impl IntoResponse {
        Json(ResponseMsg {
            msg: "ok".to_string(),
        })
    }
    pub async fn page404() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Html(include_str!("../../../../web/404.html")),
        )
    }
}
