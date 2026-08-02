mod table_basic;
pub mod tables;

use sqlx::{Pool, Postgres as SQLXPostgres};
use tables::task_comments::TaskComments;
use tables::task_histories::TaskHistories;
use tables::tasks::Tasks;
use tables::team_members::TeamMembers;
use tables::teams::Teams;
use tables::users::Users;

#[derive(Debug)]
pub struct Postgres<'a> {
    pub tbl_users: Users<'a>,
    pub tbl_teams: Teams<'a>,
    pub tbl_team_members: TeamMembers<'a>,
    pub tbl_tasks: Tasks<'a>,
    pub tbl_task_histories: TaskHistories<'a>,
    pub tbl_task_comments: TaskComments<'a>,
}

impl<'a> Postgres<'a> {
    pub fn new(pool: &'a Pool<SQLXPostgres>) -> Self {
        Self {
            tbl_users: Users::new(pool),
            tbl_teams: Teams::new(pool),
            tbl_team_members: TeamMembers::new(pool),
            tbl_tasks: Tasks::new(pool),
            tbl_task_histories: TaskHistories::new(pool),
            tbl_task_comments: TaskComments::new(pool),
        }
    }
}
