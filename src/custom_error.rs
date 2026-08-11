use http::StatusCode;
use std::error::Error;
use std::fmt;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct CustomError {
    status_code: StatusCode,
    public_msg: String,
    internal_error_msg: String,
}
impl CustomError {
    pub fn new(status_code: StatusCode, public_msg: String, internal_error_msg: String) -> Self {
        Self {
            status_code,
            public_msg,
            internal_error_msg,
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Custom error. status code - {}, public msg - {}, internal error msg - {}",
            self.status_code, self.public_msg, self.internal_error_msg
        )
    }
}

impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        (self.status_code, self.public_msg.to_string()).into_response()
    }
}

impl Error for CustomError {}

impl From<String> for CustomError {
    fn from(value: String) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            public_msg: value.clone(),
            internal_error_msg: value.clone(),
        }
    }
}

impl From<&str> for CustomError {
    fn from(value: &str) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            public_msg: value.to_string(),
            internal_error_msg: value.to_string(),
        }
    }
}

impl From<CustomError> for String {
    fn from(err: CustomError) -> Self {
        err.to_string()
    }
}
