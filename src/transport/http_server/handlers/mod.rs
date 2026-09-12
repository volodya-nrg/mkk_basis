pub mod auth;
pub mod etc;
pub mod task_comments;
pub mod tasks;
pub mod teams;
pub mod users;

use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

use crate::err_msg::ErrMsg;
use crate::transport::models::ResponseMsg;
use crate::usecase::UseCaseError;

pub struct HandlerError {
    pub source: UseCaseError,
    pub handler: &'static str,
}
impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let mut public_error_result = String::from("server internal error");
        let mut internal_error_result = String::new();
        let mut status_code_result = StatusCode::INTERNAL_SERVER_ERROR;

        match self.source {
            UseCaseError::Common(v) => {
                internal_error_result = v;
            }
            UseCaseError::ForTransport {
                status_code,
                public_err,
                internal_err,
            } => {
                status_code_result = status_code;
                public_error_result = public_err;

                if let Some(v) = internal_err {
                    internal_error_result = v;
                }
            }
            UseCaseError::UserNotExists => {
                public_error_result = ErrMsg::NotFoundUser.to_string();
                status_code_result = StatusCode::NOT_FOUND;
            }
        }

        if !internal_error_result.is_empty() {
            log::error!("{}; {}", self.handler, internal_error_result);
        }

        (
            status_code_result,
            Json(ResponseMsg {
                msg: public_error_result,
            }),
        )
            .into_response()
    }
}
