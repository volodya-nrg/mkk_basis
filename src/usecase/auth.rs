#[derive(Clone)]
pub struct Auth {
    tasks_repo: crate::adapter::db::postgres::tasks::Tasks,
}

impl Auth {
    pub fn new(tasks_repo: crate::adapter::db::postgres::tasks::Tasks) -> Self {
        Self {tasks_repo}
    }
    pub fn login(&self) -> Result<(), String> {
        println!("--- usecase auth login");
        let items = self
            .tasks_repo
            .get_all()
            .map_err(|e| format!("failed to get items: {e}"))?;
        Ok(())
    }
    pub fn logout(&self) -> Result<(), String> {
        println!("--- usecase auth logout");
        Ok(())
    }
    pub fn register(&self) -> Result<(), String> {
        println!("--- usecase auth register");
        Ok(())
    }
}
