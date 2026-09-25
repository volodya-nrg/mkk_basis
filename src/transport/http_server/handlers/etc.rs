use axum::Json;
use axum::http::StatusCode;
use axum::response::Html;

use crate::transport::models::ResponseMsg;

#[utoipa::path(
    get,
    path = "/",
    operation_id = "etc_index",
    responses(
        (status = 200, description = "Главная страница", body = String, content_type = "text/html")
    ),
    tag = "etc",
)]
pub async fn index() -> Html<&'static str> {
    Html(include_str!("../../../../web/index.html"))
}

#[utoipa::path(
    get,
    path = "/health",
    operation_id = "etc_health",
    responses(
        (status = 200, description = "Проверка 'здоровья' сервиса", body = ResponseMsg)
    ),
    tag = "etc",
)]
pub async fn health() -> Json<ResponseMsg> {
    Json(ResponseMsg {
        msg: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/404", // Или любой другой путь, на который она вешается
    operation_id = "etc_page404",
    responses(
        (status = 404, description = "Страница не найдена", body = String, content_type = "text/html")
    ),
    tag = "etc",
)]
pub async fn page404() -> (StatusCode, Html<&'static str>) {
    (
        StatusCode::NOT_FOUND,
        Html(include_str!("../../../../web/404.html")),
    )
}
