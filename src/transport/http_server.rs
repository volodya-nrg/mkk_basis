pub mod handlers;
pub mod middleware;

use crate::usecase::UseCase;
use axum::routing::{get, post, put};
use axum::{Router, middleware as AxumMiddleware};
use axum_server::tls_rustls::RustlsConfig;
use handlers::{auth, etc, tasks, teams, users};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

pub struct HTTPServer {
    addr: String,
    use_case: UseCase,
    tls_config: Option<RustlsConfig>,
}

impl HTTPServer {
    pub fn new(addr: String, use_case: UseCase, tls_config: Option<RustlsConfig>) -> Self {
        Self {
            addr,
            use_case,
            tls_config,
        }
    }
    pub async fn run(&self) -> Result<(), String> {
        let addr = SocketAddr::from_str(self.addr.as_str())
            .map_err(|e| format!("failed to create socket addr: {e}"))?;
        let router = self.get_router();

        match self.tls_config.clone() {
            Some(config) => {
                log::debug!("https-server run on {}", addr);
                axum_server::bind_rustls(addr, config) // тут внутри создается свой listener
                    .serve(router.into_make_service())
                    .await
                    .map_err(|e| format!("failed to serve(https): {e}"))?;
            }
            None => {
                let listener = TcpListener::bind(addr)
                    .await
                    .map_err(|e| format!("failed to create tcp listener: {e}"))?;
                log::debug!("http-server run on {}", addr);
                axum::serve(listener, router)
                    .await
                    .map_err(|e| format!("failed to serve(http): {e}"))?;
            }
        }

        Ok(())
    }
    fn get_router(&self) -> Router {
        let public = Router::new()
            .route("/", get(etc::Handlers::index))
            .route("/health", get(etc::Handlers::health));
        let api = Router::new()
            // auth
            .route("/api/v1/register", post(auth::Handlers::register))
            .route("/api/v1/login", post(auth::Handlers::login))
            .route("/api/v1/logout", post(auth::Handlers::logout))
            // teams
            .route(
                "/api/v1/teams",
                get(teams::Handlers::list).post(teams::Handlers::create),
            )
            .route("/api/v1/teams/{id}/invite", post(teams::Handlers::invite))
            // tasks
            .route(
                "/api/v1/tasks",
                get(tasks::Handlers::list).post(tasks::Handlers::create),
            )
            .route("/api/v1/tasks/{id}", put(tasks::Handlers::update))
            .route("/api/v1/tasks/{id}/history", get(tasks::Handlers::history))
            // users
            .route(
                "/api/v1/users",
                get(users::Handlers::list).post(users::Handlers::create),
            )
            .route(
                "/api/v1/users/{id}",
                get(users::Handlers::one)
                    .put(users::Handlers::update)
                    .delete(users::Handlers::delete),
            );
        let static_loc = Router::new()
            .nest_service("/js", ServeDir::new("./web/js"))
            .nest_service("/css", ServeDir::new("./web/css"))
            .nest_service("/images", ServeDir::new("./web/images"))
            .nest_service("/robots.txt", ServeFile::new("./web/robots.txt"))
            .nest_service("/sitemap.xml", ServeFile::new("./web/sitemap.xml"));
        Router::new()
            .merge(public)
            .merge(api)
            .merge(static_loc)
            .route_layer(AxumMiddleware::from_fn(middleware::err::err))
            .fallback(etc::Handlers::page404)
            .with_state(self.use_case.clone())
    }
}

pub fn configure_tls(
    ca_data: Vec<u8>,
    crt_data: Vec<u8>,
    key_data: Vec<u8>,
) -> Result<RustlsConfig, String> {
    let ca_certs =
        convert_pem_certificates(ca_data).map_err(|e| format!("failed to convert ca: {e}"))?;
    let crt_certs =
        convert_pem_certificates(crt_data).map_err(|e| format!("failed to convert crt: {e}"))?;
    let key = convert_private_key_from_file(key_data)
        .map_err(|e| format!("failed to convert key: {e}"))?;

    let mut root_store = RootCertStore::empty();
    ca_certs.iter().cloned().try_for_each(|item| {
        root_store
            .add(item)
            .map_err(|e| format!("failed to add certificate to store: {e}"))
    })?;

    let client_verifier = WebPkiClientVerifier::builder(root_store.into())
        .build()
        .map_err(|e| format!("failed to build verifier: {e}"))?;
    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(crt_certs, key)
        .map_err(|e| format!("failed to create server-config: {e}"))?;

    // RustlsConfig::from_pem(server_cert_str.into_bytes(), server_key_str.into_bytes()).await.unwrap();
    Ok(RustlsConfig::from_config(Arc::new(config)))
}

fn convert_pem_certificates(data: Vec<u8>) -> Result<Vec<CertificateDer<'static>>, String> {
    let items = rustls_pemfile::certs(&mut data.as_slice())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("failed to parse data: {e}"))?;
    Ok(items)
}
fn convert_private_key_from_file(data: Vec<u8>) -> Result<PrivateKeyDer<'static>, String> {
    rustls_pemfile::private_key(&mut data.as_slice())
        .map_err(|e| format!("failed to parse data: {e}"))?
        .ok_or_else(|| "no private key found".to_string())
}
