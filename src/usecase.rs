mod helpers;
mod mapper;

pub mod auth;
pub mod models;
pub mod task_comments;
pub mod tasks;
pub mod teams;
pub mod users;

use http::StatusCode;
use std::sync::Arc;

use crate::adapter::{
    db::{errors::RepositoryError, storage::Storage},
    email::{ConfirmationCodeStorer, EmailSender},
    jwt::{JWTError, Jwt as JWTService},
};
use crate::app_errors::AppErr;

#[derive(Clone)] // clone из-за axum
pub struct UseCase {
    pub auth: auth::Auth,
    pub teams: teams::Teams,
    pub tasks: tasks::Tasks,
    pub task_comments: task_comments::TaskComments,
    pub users: users::Users,
}

// внутри нельзя менять состояние (&mut self)
impl UseCase {
    pub fn new(
        addr: String,
        storage: Arc<dyn Storage>,
        jwt_service: JWTService,
        email_sender: Arc<dyn EmailSender>,
        code_store: Arc<dyn ConfirmationCodeStorer>,
    ) -> Self {
        Self {
            auth: auth::Auth::new(addr, jwt_service, email_sender, code_store, storage.clone()),
            teams: teams::Teams::new(storage.clone()),
            tasks: tasks::Tasks::new(storage.clone()),
            task_comments: task_comments::TaskComments::new(storage.clone()),
            users: users::Users::new(storage.clone()),
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
