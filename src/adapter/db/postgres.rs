mod table_basic;
pub mod tables;

use sqlx::{Pool, Postgres as SQLXPostgres};

use tables::{
    task_comments::TaskComments, task_histories::TaskHistories, tasks::Tasks,
    team_members::TeamMembers, teams::Teams, users::Users,
};

// #[allow(dead_code), derive(Clone)]
#[derive(Clone)] // даем возможность клонирования для тестов
pub struct Postgres {
    pub tbl_users: Users,
    pub tbl_teams: Teams,
    pub tbl_team_members: TeamMembers,
    pub tbl_tasks: Tasks,
    pub tbl_task_histories: TaskHistories,
    pub tbl_task_comments: TaskComments,
}

impl Postgres {
    pub fn new(pool: Pool<SQLXPostgres>) -> Self {
        Self {
            // state от axum необходимо статическим, поэтому ссылку на pool тут не передаем
            tbl_users: Users::new(pool.clone()),
            tbl_teams: Teams::new(pool.clone()),
            tbl_team_members: TeamMembers::new(pool.clone()),
            tbl_tasks: Tasks::new(pool.clone()),
            tbl_task_histories: TaskHistories::new(pool.clone()),
            tbl_task_comments: TaskComments::new(pool.clone()),
        }
    }
}
