use axum::{extract::Request, middleware::Next, response::Response};

pub async fn err(req: Request, next: Next) -> Response {
    let mut resp = next.run(req).await;
    let status_code = resp.status();

    if status_code.is_server_error() {
        let (parts, body) = resp.into_parts(); // let body = resp.into_body();
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .unwrap_or_default();

        log::error!("err middleware: {}, {:?}", status_code, body_bytes);

        resp = Response::from_parts(parts, body_bytes.into());
    }

    resp
}
