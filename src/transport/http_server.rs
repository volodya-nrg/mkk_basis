pub mod handlers;
pub mod middleware;

use axum::{
    extract::DefaultBodyLimit,
    middleware as AxumMiddleware,
    routing::{delete, get, post},
};
use axum_server::Handle;
use axum_server::tls_rustls::RustlsConfig;
use http::StatusCode;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    normalize_path::NormalizePathLayer,
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapter::email::EmailSender;
use crate::usecase::UseCase;

use handlers::{auth, etc, task_comments, tasks, teams, users};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("access_token"))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(info(description = "Сервис предоставляет api"))]
#[openapi(
    paths(
        auth::register,
        auth::register_confirm,
        auth::login,
        auth::logout,
        auth::refresh_tokens,
        //
        etc::index,
        etc::health,
        etc::page404,
        //
        task_comments::list,
        task_comments::create,
        task_comments::delete,
        //
        tasks::list,
        tasks::one,
        tasks::create,
        tasks::update,
        tasks::delete,
        tasks::history,
        //
        teams::list,
        teams::one,
        teams::create,
        teams::update,
        teams::delete,
        teams::invite,
        //
        users::list,
        users::one,
        users::create,
        users::update,
        users::delete,
    ),
    modifiers(&SecurityAddon),
)]
struct MyApiDoc;

pub struct HTTPServer<T> {
    addr: String,
    use_case: UseCase<T>,
    tls_config: Option<RustlsConfig>,
}

impl<T: EmailSender> HTTPServer<T> {
    pub const fn new(addr: String, use_case: UseCase<T>, tls_config: Option<RustlsConfig>) -> Self {
        Self {
            addr,
            use_case,
            tls_config,
        }
    }
    pub async fn run(&self) -> Result<(), String> {
        let addr = SocketAddr::from_str(self.addr.as_str())
            .map_err(|e| format!("failed to create socket addr: {e}"))?;
        let (router, api) = OpenApiRouter::with_openapi(MyApiDoc::openapi())
            .merge(self.get_router())
            .split_for_parts();
        let app = router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));

        match self.tls_config.clone() {
            Some(config) => {
                let handle = Handle::new();
                let shutdown_handle = handle.clone();
                tokio::spawn(async move {
                    shutdown_signal().await;
                    // Указываем таймаут, чтобы не ждать зависшие соединения вечно.
                    // None означает бесконечное ожидание.
                    shutdown_handle.graceful_shutdown(Some(Duration::from_secs(10)));
                });

                log::debug!("https-server run on {}", addr);
                axum_server::bind_rustls(addr, config)
                    .handle(handle)
                    .serve(app.into_make_service())
                    .await
                    .map_err(|e| format!("failed to serve(https): {e}"))?;
            }
            None => {
                let listener = TcpListener::bind(addr)
                    .await
                    .map_err(|e| format!("failed to create tcp listener: {e}"))?;
                log::debug!("http-server run on {}", addr);
                axum::serve(listener, app)
                    .with_graceful_shutdown(shutdown_signal())
                    .await
                    .map_err(|e| format!("failed to serve(http): {e}"))?;
            }
        }

        Ok(())
    }
    fn get_router(&self) -> OpenApiRouter {
        let layer_auth =
            AxumMiddleware::from_fn_with_state(self.use_case.clone(), middleware::auth::auth);
        let public = OpenApiRouter::new()
            //.route_service("/", ServeFile::new("../../web/index.html")) - это не вариант
            .route("/", get(etc::index))
            .route("/health", get(etc::health))
            .route("/register/confirm", get(auth::register_confirm));
        let api = OpenApiRouter::new()
            // auth
            .route("/api/v1/register", post(auth::register))
            .route("/api/v1/login", post(auth::login))
            .route(
                "/api/v1/logout",
                post(auth::logout).layer(layer_auth.clone()), // проверка на auth все равно стоит
            )
            .route("/api/v1/refresh_tokens", post(auth::refresh_tokens))
            // teams
            .route(
                "/api/v1/teams",
                get(teams::list)
                    .post(teams::create)
                    .layer(layer_auth.clone()),
            )
            .route(
                "/api/v1/teams/{id}",
                get(teams::one)
                    .put(teams::update)
                    .delete(teams::delete)
                    .layer(layer_auth.clone()),
            )
            .route(
                "/api/v1/teams/{id}/invite",
                post(teams::invite).layer(layer_auth.clone()),
            )
            // tasks
            .route(
                "/api/v1/tasks",
                get(tasks::list)
                    .post(tasks::create)
                    .layer(layer_auth.clone()),
            )
            .route(
                "/api/v1/tasks/{id}",
                get(tasks::one)
                    .put(tasks::update)
                    .delete(tasks::delete)
                    .layer(layer_auth.clone()),
            )
            .route(
                "/api/v1/tasks/{id}/history",
                get(tasks::history).layer(layer_auth.clone()),
            )
            .route(
                "/api/v1/tasks/{id}/comments",
                get(task_comments::list)
                    .post(task_comments::create)
                    .layer(layer_auth.clone()),
            )
            .route(
                "/api/v1/tasks/comment/{id}",
                delete(task_comments::delete).layer(layer_auth.clone()),
            )
            // users
            .route(
                "/api/v1/users",
                get(users::list)
                    .post(users::create)
                    .layer(layer_auth.clone()),
            )
            .route(
                "/api/v1/users/{id}",
                get(users::one)
                    .patch(users::update)
                    .delete(users::delete)
                    .layer(layer_auth),
            );
        let static_loc = OpenApiRouter::new()
            .nest_service("/js", ServeDir::new("./web/js"))
            .nest_service("/css", ServeDir::new("./web/css"))
            .nest_service("/images", ServeDir::new("./web/images"))
            .nest_service("/robots.txt", ServeFile::new("./web/robots.txt"))
            .nest_service("/sitemap.xml", ServeFile::new("./web/sitemap.xml"));
        /*
        Router::new()
            .route_service("/js", ServeDir::new("./web/js"))
            .route_service("/css", ServeDir::new("./web/css"))
            .route_service("/images", ServeDir::new("./web/images"))
            .route_service("/robots.txt", ServeFile::new("./web/robots.txt"))
            .route_service("/sitemap.xml", ServeFile::new("./web/sitemap.xml"));
        */
        /*
        example:
            let cors = CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_credentials(true)
            .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);
        */
        let cors = CorsLayer::new()
            // .allow_origin(Any)
            // .allow_methods(Any)
            // .allow_headers(Any)
            .allow_credentials(true); // нужно для куки
        OpenApiRouter::new()
            .merge(public)
            .merge(api)
            .merge(static_loc)
            .route_layer(AxumMiddleware::from_fn(middleware::err::err))
            .layer(cors)
            .layer(NormalizePathLayer::trim_trailing_slash())
            .layer(DefaultBodyLimit::disable()) // надо именно выключить
            .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 20MB лимит. Если будет больше, то обработка multipart сервером выдаст ошибку.
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(10),
            ))
            .fallback(etc::page404)
            .with_state(self.use_case.clone())
    }
}

pub fn configure_tls(
    ca_data: Vec<u8>,
    crt_data: Vec<u8>,
    key_data: Vec<u8>,
) -> Result<RustlsConfig, String> {
    // эта штука нужна что определения крипто-провайдера
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
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
#[cfg(unix)]
async fn wait_for_signal(kind: signal::unix::SignalKind) {
    match signal::unix::signal(kind) {
        Ok(mut stream) => {
            stream.recv().await;
        }
        Err(e) => {
            log::error!("Failed to install signal handler {kind:?}: {e}");
            std::future::pending::<()>().await; // не завершаем работу ложно — просто ждем
        }
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        tokio::select! {
            _ = wait_for_signal(signal::unix::SignalKind::interrupt()) => log::info!("Received SIGINT"),
            _ = wait_for_signal(signal::unix::SignalKind::terminate()) => log::info!("Received SIGTERM"),
        }
    }
    #[cfg(not(unix))]
    {
        signal::ctrl_c().await.ok();
        info!("Received Ctrl+C");
    }
}
