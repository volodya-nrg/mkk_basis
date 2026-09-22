use axum::Json;
use axum::http::StatusCode;
use axum::response::Html;
use serde_json::{Value, json};

use crate::transport::models::ResponseMsg;

pub struct Handlers {}

impl Handlers {
    pub async fn index() -> Html<&'static str> {
        Html(include_str!("../../../../web/index.html"))
    }
    pub async fn health() -> Json<Value> {
        Json(json!(ResponseMsg {
            msg: "ok".to_string(),
        }))
    }
    pub async fn page404() -> (StatusCode, Html<&'static str>) {
        (
            StatusCode::NOT_FOUND,
            Html(include_str!("../../../../web/404.html")),
        )
    }
}
