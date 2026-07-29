pub mod auth;
pub mod models;
pub mod tasks;
pub mod teams;

use crate::adapter::db::postgres::Postgres;
use auth::Auth;
use tasks::Tasks;
use teams::Teams;

#[derive(Clone)]
pub struct UseCase {
    pub auth: Auth,
    pub tasks: Tasks,
    pub teams: Teams,
}

impl UseCase {
    pub fn new(postgres: Postgres) -> Self {
        Self {
            auth: Auth::new(postgres.tbl_tasks.clone()),
            tasks: Tasks::new(postgres.tbl_tasks.clone()),
            teams: Teams::new(postgres.tbl_teams.clone()),
        }
    }
}
