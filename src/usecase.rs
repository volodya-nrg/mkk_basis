mod helpers;
mod mapper;

pub mod auth;
pub mod models;
pub mod task_comments;
pub mod tasks;
pub mod teams;
pub mod users;

use crate::adapter::{
    db::{
        errors::RepositoryError,
        postgres::{Postgres, transactor::TransactionError},
    },
    email::EmailSender,
    jwt::JWTError,
    jwt::Jwt as JWTService,
};
use crate::app_errors::AppErr;
use http::StatusCode;
use std::sync::Arc;

#[derive(Clone)] // clone из-за axum
pub struct UseCase {
    pub auth: auth::Auth,
    pub teams: teams::Teams,
    pub tasks: tasks::Tasks,
    pub task_comments: task_comments::TaskComments,
    pub users: users::Users,
}

impl UseCase {
    pub fn new(
        addr: String,
        db: Arc<Postgres>,
        jwt_service: JWTService,
        email_sender: Arc<dyn EmailSender>,
    ) -> Self {
        Self {
            auth: auth::Auth::new(
                addr,
                jwt_service,
                email_sender,
                db.transactor.clone(),
                db.tbl_users.clone(),
            ),
            teams: teams::Teams::new(
                db.transactor.clone(),
                db.tbl_teams.clone(),
                db.tbl_team_members.clone(),
            ),
            tasks: tasks::Tasks::new(
                db.transactor.clone(),
                db.tbl_tasks.clone(),
                db.tbl_task_histories.clone(),
                db.tbl_team_members.clone(),
            ),
            task_comments: task_comments::TaskComments::new(
                db.transactor.clone(),
                db.tbl_task_comments.clone(),
            ),
            users: users::Users::new(db.transactor.clone(), db.tbl_users.clone()),
        }
    }
}

pub enum UseCaseError {
    Common(String),
    Transport {
        status_code: StatusCode,
        public_err: String,
        internal_err: Option<String>,
    },
    UserNotExists,
}

/*
Чтобы from не делать, можно короче написать с помощью thiserror.
Эта аннотация говорит thiserror, что нужно сгенерировать соответствующую реализацию трэйта From.
#[derive(Debug, Error)]
enum PurchaseError {
    #[error("Nested servation error: (0)")]
    ReservationFailed(#[from] ReserveError)
    #[error("Nested shipping error: (0)")]
    ShippingFailed(#[from] ShipmentError)
}
*/

impl From<RepositoryError> for UseCaseError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFoundRow => Self::Transport {
                status_code: StatusCode::NOT_FOUND,
                public_err: AppErr::NotFoundItem.to_string(),
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
