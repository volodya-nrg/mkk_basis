pub mod handlers;
pub mod middleware;

use crate::usecase::UseCase;
use axum::routing::{get, post, put};
use axum::{Router, middleware as AxumMiddleware};
use handlers::{auth, etc, tasks, teams};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

pub struct HTTPServer {
    addr: String,
    use_case: UseCase,
}

impl HTTPServer {
    pub fn new(addr: String, use_case: UseCase) -> Self {
        Self { addr, use_case }
    }
    pub async fn run(&self) -> Result<(), String> {
        let router = self.get_router();
        let listener = TcpListener::bind(self.addr.as_str())
            .await
            .map_err(|e| format!("failed to bind addr: {e}"))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| format!("failed to serve: {e}"))?;

        Ok(())
    }
    fn get_router(&self) -> Router {
        Router::new()
            // index
            .route("/", get(etc::Handlers::index))
            .route("/healthz", get(etc::Handlers::healthz))
            // auth
            .route("/api/v1/register", post(auth::Handlers::register))
            .route("/api/v1/login", post(auth::Handlers::login))
            .route("/api/v1/logout", post(auth::Handlers::logout))
            // teams
            .route(
                "/api/v1/teams",
                get(teams::Handlers::teams_list).post(teams::Handlers::teams_create),
            )
            .route(
                "/api/v1/teams/{id}/invite",
                post(teams::Handlers::teams_invite),
            )
            // tasks
            .route(
                "/api/v1/tasks",
                get(tasks::Handlers::tasks_list).post(tasks::Handlers::tasks_create),
            )
            .route("/api/v1/tasks/{id}", put(tasks::Handlers::tasks_update))
            .route(
                "/api/v1/tasks/{id}/history",
                get(tasks::Handlers::tasks_history),
            )
            // static
            .nest_service("/js", ServeDir::new("./web/js"))
            .nest_service("/css", ServeDir::new("./web/css"))
            .nest_service("/images", ServeDir::new("./web/images"))
            .nest_service("/robots.txt", ServeFile::new("./web/robots.txt"))
            .nest_service("/sitemap.xml", ServeFile::new("./web/sitemap.xml"))
            // middleware
            .route_layer(AxumMiddleware::from_fn(middleware::err::err))
            // other
            .fallback(etc::Handlers::page404)
            .with_state(self.use_case.clone())
    }
}
