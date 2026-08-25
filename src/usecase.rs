pub mod auth;
mod helpers;
mod mapper;
pub mod models;
pub mod task_comments;
pub mod tasks;
pub mod teams;
pub mod users;

use crate::adapter::{
    db::postgres::Postgres as PostgresService, email::EmailSender, jwt::Jwt as JWTService,
};
use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use thiserror::Error as ThisError;

#[derive(Clone)] // из-за axum-state
pub struct UseCase<ES: EmailSender> {
    pub auth: auth::Auth<ES>,
    pub tasks: tasks::Tasks,
    pub task_comments: task_comments::TaskComments,
    pub teams: teams::Teams,
    pub users: users::Users,
}

impl<ES: EmailSender> UseCase<ES> {
    pub fn new(
        addr: String,
        postgres: PostgresService,
        jwt_service: JWTService,
        email_sender: ES,
    ) -> Self {
        Self {
            auth: auth::Auth::new(addr, postgres.tbl_users.clone(), jwt_service, email_sender),
            tasks: tasks::Tasks::new(postgres.tbl_tasks, postgres.tbl_task_histories),
            task_comments: task_comments::TaskComments::new(postgres.tbl_task_comments),
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
