use std::sync::Arc;

use crate::adapter::{
    db::{storage::Storage},
    email::{ConfirmationCodeStorer, EmailSender},
    jwt::{Jwt as JWTService},
};
use crate::usecase::{auth, task_comments, tasks, teams, users};

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
