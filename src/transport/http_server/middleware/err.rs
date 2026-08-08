use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub async fn err(req: Request, next: Next) -> Result<Response, StatusCode> {
    let resp = next.run(req).await;
    Ok(resp)
}
