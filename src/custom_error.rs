use http::StatusCode;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct CustomError {
    original_error: String,
    status_code: StatusCode,
}
impl CustomError {
    pub fn new(status_code: StatusCode, original_error: String) -> Self {
        Self {
            original_error,
            status_code,
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "custom error: {}, {}",
            self.status_code, self.original_error
        )
    }
}

impl Error for CustomError {}

impl From<String> for CustomError {
    fn from(value: String) -> Self {
        Self {
            original_error: value,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<&str> for CustomError {
    fn from(value: &str) -> Self {
        Self {
            original_error: value.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<CustomError> for String {
    fn from(err: CustomError) -> Self {
        err.to_string()
    }
}
