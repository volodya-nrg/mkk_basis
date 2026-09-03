mod helpers;
mod mapper;

pub mod auth;
pub mod models;
pub mod task_comments;
pub mod tasks;
pub mod teams;
pub mod users;

use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use thiserror::Error as ThisError;

use crate::adapter::db::postgres::Postgres;
use crate::adapter::{db::errors::RepositoryError, email::EmailSender, jwt::Jwt as JWTService};
use crate::err_msg::ErrMsg;

#[derive(Clone)] // из-за axum-state
pub struct UseCase<ES> {
    pub auth: auth::Auth<ES>,
    pub teams: teams::Teams,
    pub tasks: tasks::Tasks,
    pub task_comments: task_comments::TaskComments,
    pub users: users::Users,
}

impl<ES> UseCase<ES>
where
    ES: EmailSender,
{
    pub fn new(addr: String, db: Postgres, jwt_service: JWTService, email_sender: ES) -> Self {
        Self {
            auth: auth::Auth::new(addr, db.tbl_users.clone(), jwt_service, email_sender),
            teams: teams::Teams::new(db.tbl_teams, db.tbl_team_members.clone()),
            tasks: tasks::Tasks::new(db.tbl_tasks, db.tbl_task_histories, db.tbl_team_members),
            task_comments: task_comments::TaskComments::new(db.tbl_task_comments),
            users: users::Users::new(db.tbl_users),
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
impl From<RepositoryError> for UseCaseError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFoundRow => UseCaseError::ForTransport {
                status_code: StatusCode::NOT_FOUND,
                public_err: ErrMsg::NotFoundItem.to_string(),
                internal_err: None,
            },
            other => UseCaseError::Common(other.to_string()),
        }
    }
}
