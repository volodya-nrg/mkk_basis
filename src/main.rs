mod adapter;
mod consts;
mod err_msg;
mod transport;
mod usecase;

use crate::adapter::jwt::Jwt;
use adapter::{config::Config, db::postgres::Postgres, logger};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use transport::http_server::HTTPServer;
use usecase::UseCase;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "./config.yaml")]
    config: String,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run(&Args::parse().config).await {
        log::error!("failed to run app: {e}");
        std::process::exit(1);
    }
}

async fn run(config_filepath: &str) -> Result<(), String> {
    let cfg = Config::new(config_filepath).map_err(|e| {
        let str = format!("failed to create new config: {e}");
        eprint!("{}", str);
        str
    })?;

    logger::init(
        &cfg.service_name,
        &cfg.version,
        &cfg.log.level,
        &cfg.log.filepath.unwrap_or("".to_string()),
        false,
    )
    .map_err(|e| format!("failed to init logger: {e}"))?;

    let private_key_bytes = std::fs::read(cfg.private_key_path)
        .map_err(|e| format!("failed to read private key: {e}"))?;
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::new(1, 0))
        .connect(&cfg.postgres.dsn)
        .await
        .map_err(|e| format!("failed to connect on DB: {e}"))?;
    let http_server = HTTPServer::new(
        cfg.http_server.address.clone(),
        UseCase::new(
            Postgres::new(pool),
            Jwt::new(private_key_bytes, 60 * 20, 60 * 60 * 24),
        ),
    );

    log::info!("http-server start on {}", cfg.http_server.address);
    http_server
        .run()
        .await
        .map_err(|e| format!("failed to run server: {e}"))
}
