pub mod auth;
mod mapper;
pub mod models;
pub mod tasks;
pub mod teams;

use crate::adapter::db::postgres::Postgres;
use auth::Auth;
use tasks::Tasks;
use teams::Teams;

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