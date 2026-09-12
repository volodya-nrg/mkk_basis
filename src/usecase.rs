mod helpers;
mod mapper;

pub mod auth;
pub mod models;
pub mod task_comments;
pub mod tasks;
pub mod teams;
pub mod users;

use http::StatusCode;
use thiserror::Error as ThisError;

use crate::adapter::{
    db::{errors::RepositoryError, postgres::Postgres, postgres::transactor::Transactor},
    email::EmailSender,
    jwt::JWTError,
    jwt::Jwt as JWTService,
};
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
    pub fn new(
        addr: String,
        db: Postgres,
        jwt_service: JWTService,
        email_sender: ES,
        transactor: Transactor,
    ) -> Self {
        Self {
            auth: auth::Auth::new(
                addr,
                jwt_service,
                email_sender,
                transactor.clone(),
                db.tbl_users.clone(),
            ),
            teams: teams::Teams::new(
                transactor.clone(),
                db.tbl_teams,
                db.tbl_team_members.clone(),
            ),
            tasks: tasks::Tasks::new(
                transactor.clone(),
                db.tbl_tasks,
                db.tbl_task_histories,
                db.tbl_team_members,
            ),
            task_comments: task_comments::TaskComments::new(
                transactor.clone(),
                db.tbl_task_comments,
            ),
            users: users::Users::new(transactor.clone(), db.tbl_users),
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
    #[error("user not found")]
    UserNotExists,
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
impl From<JWTError> for UseCaseError {
    fn from(e: JWTError) -> Self {
        match e {
            JWTError::ExpiredToken => UseCaseError::Common(e.to_string()), // пусть явно стоит
            other => UseCaseError::Common(other.to_string()),
        }
    }
}
