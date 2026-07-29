mod handlers;

use crate::usecase::UseCase;
use axum::Router;
use axum::routing::get;
use handlers::Handlers;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub use_case: Arc<UseCase>,
}
impl AppState {
    pub fn new(use_case: UseCase) -> Self {
        Self {
            use_case: Arc::new(use_case),
        }
    }
}

pub struct HTTPServer {}

impl HTTPServer {
    pub async fn run(addr: &str, use_case: UseCase) -> Result<(), String> {
        let state = Arc::new(AppState::new(use_case));
        let router = Router::new()
            .route("/api/v1/login", get(Handlers::login))
            .route("/api/v1/register", get(Handlers::register))
            .route("/api/v1/tasks", get(Handlers::tasks))
            .route("/api/v1/teams", get(Handlers::teams))
            .with_state(state);
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| format!("failed to bind addr: {e}"))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| format!("failed to serve: {e}"))?;

        Ok(())
    }
}
