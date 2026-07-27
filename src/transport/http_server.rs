mod handlers;

use crate::usecase::UseCase;
use axum::Router;
use axum::routing::get;
use handlers::Handlers;
use tokio::net::TcpListener;

pub struct HTTPServer {
    addr: String,
    router: Router,
}

impl HTTPServer {
    pub fn new(addr: String, use_case: UseCase) -> Self {
        let handlers = Handlers::new(use_case);
        let router = Router::new()
            .route("/api/v1/login", get(Handlers::login))
            .route("/api/v1/register", get(Handlers::register))
            .route("/api/v1/tasks", get(Handlers::tasks))
            .route("/api/v1/teams", get(Handlers::teams));

        Self { addr, router }
    }
    pub async fn run(self) -> Result<(), String> {
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| format!("failed to bind addr: {e}"))?;

        axum::serve(listener, self.router)
            .await
            .map_err(|e| format!("failed to serve: {e}"))?;
        
        Ok(())
    }
}
