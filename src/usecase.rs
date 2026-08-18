pub mod auth;
mod mapper;
pub mod models;
pub mod tasks;
pub mod teams;
pub mod users;
mod helpers;

use crate::adapter::{db::postgres::Postgres as PostgresService, jwt::Jwt as JWTService};
use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use thiserror::Error as ThisError;

#[derive(Clone)] // из-за axum-state
pub struct UseCase {
    pub auth: auth::Auth,
    pub tasks: tasks::Tasks,
    pub teams: teams::Teams,
    pub users: users::Users,
}

impl UseCase {
    pub fn new(postgres: PostgresService, jwt_service: JWTService) -> Self {
        Self {
            auth: auth::Auth::new(postgres.tbl_users.clone(), jwt_service),
            tasks: tasks::Tasks::new(postgres.tbl_tasks, postgres.tbl_task_histories),
            teams: teams::Teams::new(postgres.tbl_teams, postgres.tbl_team_members),
            users: users::Users::new(postgres.tbl_users),
        }
    }
}

// ------

#[derive(ThisError, Debug)]
pub enum UseCaseError {
    #[error("{0}")]
    Common(String),
    #[error(
        "{status_code}; {public_err}; {internal_err};",
        internal_err = internal_err.as_deref().unwrap_or("none")
    )]
    ForTransport {
        status_code: StatusCode,
        public_err: String,
        internal_err: Option<String>,
    },
}
impl IntoResponse for UseCaseError {
    fn into_response(self) -> Response {
        let mut public_error_result = String::from("Server internal error");
        let mut internal_error_result = String::new();
        let mut status_code_result = StatusCode::INTERNAL_SERVER_ERROR;

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

                if let Some(v) = internal_err {
                    internal_error_result = v;
                }
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
