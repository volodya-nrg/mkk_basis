pub mod auth;
mod mapper;
pub mod models;
pub mod tasks;
pub mod teams;

use crate::adapter::db::postgres::Postgres;
use auth::Auth;
use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use tasks::Tasks;
use teams::Teams;
use thiserror::Error;

#[derive(Clone)] // из-за axum-state
pub struct UseCase {
    pub auth: Auth,
    pub tasks: Tasks,
    pub teams: Teams,
}

impl UseCase {
    pub fn new(postgres: Postgres) -> Self {
        Self {
            auth: Auth::new(postgres.tbl_users),
            tasks: Tasks::new(postgres.tbl_tasks, postgres.tbl_task_histories),
            teams: Teams::new(postgres.tbl_teams, postgres.tbl_team_members),
        }
    }
}

// ------

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("{0}")]
    Common(String),
    #[error("{status_code}; {public_err}; {internal_err};")]
    ForTransport {
        status_code: StatusCode,
        public_err: String,
        internal_err: String,
    },
}
impl IntoResponse for UseCaseError {
    fn into_response(self) -> Response {
        let mut public_error_result = String::from("Server internal error");
        let internal_error_result;
        let mut status_code_result: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

        match self {
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
                internal_error_result = internal_err;
            }
        }

        if !internal_error_result.is_empty() {
            log::error!("{}", internal_error_result);
        }

        (
            status_code_result,
            Json(json!({"message": public_error_result})),
        )
            .into_response()
    }
}
