use axum::{extract::Request, middleware::Next, response::Response};

pub async fn err(req: Request, next: Next) -> Response {
    next.run(req).await
}
