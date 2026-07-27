pub mod tasks;
pub mod teams;

use sqlx::postgres::PgPoolOptions;
use tasks::Tasks;
use teams::Teams;

pub struct Postgres {
    pub tbl_tasks: Tasks,
    pub tbl_teams: Teams,
}

impl Postgres {
    pub async fn new(dsn: String) -> Result<Self, String> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(dsn.as_str())
            .await
            .map_err(|e| format!("failed to connect to db: {e}"))?;

        Ok(Self {
            tbl_tasks: Tasks::new(pool.clone()),
            tbl_teams: Teams::new(pool.clone()),
        })
    }
}
