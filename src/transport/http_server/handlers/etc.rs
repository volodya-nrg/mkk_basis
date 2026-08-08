use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

pub struct Handlers {}

impl Handlers {
    pub async fn index() -> impl IntoResponse {
        Html(include_str!("../../../../web/index.html"))
    }
    pub async fn healthz() -> impl IntoResponse {
        //()
        StatusCode::OK
    }
    pub async fn page404() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Html(include_str!("../../../../web/404.html")),
        )
    }
}
