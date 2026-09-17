mod helpers;
mod mapper;

pub mod auth;
pub mod models;
pub mod task_comments;
pub mod tasks;
pub mod teams;
pub mod users;

use http::StatusCode;

use crate::adapter::db::postgres::transactor::TransactionError;
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
            users: users::Users::new(transactor, db.tbl_users),
        }
    }
}

// ------

// тут thiserror не нужен, т.к. конвертация в строку ни где не происходит, а берутся их значения
#[derive(Debug)]
pub enum UseCaseError {
    Common(String),
    Transport {
        status_code: StatusCode,
        public_err: String,
        internal_err: Option<String>,
    },
    UserNotExists,
}
impl From<RepositoryError> for UseCaseError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFoundRow => Self::Transport {
                status_code: StatusCode::NOT_FOUND,
                public_err: ErrMsg::NotFoundItem.to_string(),
                internal_err: None,
            },
            other => Self::Common(other.to_string()),
        }
    }
}
impl From<JWTError> for UseCaseError {
    fn from(e: JWTError) -> Self {
        Self::Common(e.to_string())
    }
}
impl From<sqlx::Error> for UseCaseError {
    fn from(e: sqlx::Error) -> Self {
        Self::Common(e.to_string())
    }
}
impl<E> From<TransactionError<E>> for UseCaseError
where
    E: Into<Self>,
{
    fn from(e: TransactionError<E>) -> Self {
        match e {
            TransactionError::Database(sqlx_err) => Self::Common(sqlx_err.to_string()),
            TransactionError::Operation(e) => e.into(),
            // other => Self::from(other), - тут было переполнение стека
        }
    }
}
