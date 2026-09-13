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

macro_rules! handler_err {
    ($source:expr) => {
        HandlerError {
            use_case_err: $source,
            source: concat!(file!(), ":", line!()), // format! тут не подойдет
        }
    };
}
pub(crate) use handler_err;

pub struct HandlerError {
    use_case_err: UseCaseError,
    source: &'static str,
}
impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let mut public_error_result = String::from("server internal error");
        let mut internal_error_result = String::new();
        let mut status_code_result = StatusCode::INTERNAL_SERVER_ERROR;

        match self.use_case_err {
            UseCaseError::Common(v) => {
                internal_error_result = v;
            }
            UseCaseError::Transport {
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
            log::error!(
                "{} ({}) -> {}",
                self.source,
                status_code_result.as_u16(),
                internal_error_result
            );
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
