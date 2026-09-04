mod adapter;
mod consts;
mod err_msg;
mod transport;
mod usecase;

use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::fs;
use std::process;
use std::time::Duration;

use adapter::{
    config::Config, db::postgres::Postgres as PostgresService, email::Email as EmailService,
    jwt::Jwt as JWTService, logger,
};
use transport::http_server::HTTPServer;
use usecase::UseCase;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "./data/config.yaml")]
    config: String,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run(&Args::parse().config).await {
        log::error!("failed to run app: {e}");
        process::exit(1);
    }
}

async fn run(config_filepath: &str) -> Result<(), String> {
    let cfg = Config::new(config_filepath).map_err(|e| {
        let str = format!("failed to create new config: {e}");
        eprint!("{}", str);
        str
    })?;

    logger::init(
        cfg.service_name,
        cfg.version,
        cfg.log.level,
        cfg.log.filepath,
        false,
    )
    .map_err(|e| format!("failed to init logger: {e}"))?;

    let private_key_bytes =
        fs::read(cfg.private_key_path).map_err(|e| format!("failed to read private key: {e}"))?;
    let mut tls_config_for_server: Option<RustlsConfig> = None;

    if cfg.http_server.tls.is_use {
        let ca_bytes = fs::read(cfg.http_server.tls.ca_filepath)
            .map_err(|e| format!("failed to read ca file: {e}"))?;
        let crt_bytes = fs::read(cfg.http_server.tls.crt_filepath)
            .map_err(|e| format!("failed to read crt file: {e}"))?;
        let key_bytes = fs::read(cfg.http_server.tls.key_filepath)
            .map_err(|e| format!("failed to read key file: {e}"))?;
        let tls_config = transport::http_server::configure_tls(ca_bytes, crt_bytes, key_bytes)
            .map_err(|e| format!("failed to configure tls: {e}"))?;
        tls_config_for_server = Some(tls_config);
    }

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::new(3, 0))
        .connect(&cfg.postgres.dsn)
        .await
        .map_err(|e| format!("failed to connect on DB: {e}"))?;
    let http_server = HTTPServer::new(
        cfg.http_server.address.clone(),
        UseCase::new(
            cfg.addr,
            PostgresService::new(pool),
            JWTService::new(private_key_bytes, 60 * 20, 60 * 60 * 24),
            EmailService::new(
                cfg.email.host,
                cfg.email.login,
                cfg.email.pass,
                cfg.email.from_email,
                cfg.email.from_name,
                Duration::from_secs(3),
            ),
        ),
        tls_config_for_server.clone(),
    );

    http_server
        .run()
        .await
        .map_err(|e| format!("failed to run server: {e}"))
}
