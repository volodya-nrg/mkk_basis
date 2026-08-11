use axum::{extract::Request, middleware::Next, response::Response};

pub async fn err(req: Request, next: Next) -> Response {
    let resp = next.run(req).await;
    resp
}
