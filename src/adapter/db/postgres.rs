pub mod tables;
pub mod transactor;

use tables::{
    task_comments::TaskComments, task_histories::TaskHistories, tasks::Tasks,
    team_members::TeamMembers, teams::Teams, users::Users,
};

#[derive(Clone, Default)] // клонирование нужно для транспортного теста
pub struct Postgres {
    pub tbl_users: Users,
    pub tbl_teams: Teams,
    pub tbl_team_members: TeamMembers,
    pub tbl_tasks: Tasks,
    pub tbl_task_histories: TaskHistories,
    pub tbl_task_comments: TaskComments,
}

impl Postgres {
    pub fn new() -> Self {
        Self {
            tbl_users: Users::new(),
            tbl_teams: Teams::new(),
            tbl_team_members: TeamMembers::new(),
            tbl_tasks: Tasks::new(),
            tbl_task_histories: TaskHistories::new(),
            tbl_task_comments: TaskComments::new(),
        }
    }
}
