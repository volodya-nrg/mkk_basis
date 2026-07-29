mod adapter;
mod transport;
mod usecase;

use adapter::{Config, Postgres, logger};
use clap::Parser;
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
        eprint!("{}", &str);
        str
    })?;

    logger::init(
        &cfg.service_name,
        &cfg.version,
        &cfg.log.level,
        &cfg.log.filepath.unwrap_or("".to_string()),
    )
    .map_err(|e| format!("failed to init logger: {e}"))?;

    let postgres = Postgres::new(&cfg.postgres.dsn)
        .await
        .map_err(|e| format!("failed to create new postgres: {e}"))?;

    log::info!("start server on {}", cfg.http_server.address);

    HTTPServer::run(&cfg.http_server.address, UseCase::new(postgres))
        .await
        .map_err(|e| format!("failed to run server: {e}"))?;

    Ok(())
}
