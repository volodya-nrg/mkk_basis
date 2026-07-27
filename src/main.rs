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

async fn run(config_filepath: &String) -> Result<(), String> {
    let cfg =
        Config::new(config_filepath).map_err(|e| format!("failed to create new config: {e}"))?;

    logger::init(cfg.service_name, cfg.version, cfg.log.level);

    let postgres = Postgres::new(cfg.postgres.dsn)
        .await
        .map_err(|e| format!("failed to create new postgres: {e}"))?;
    let use_case = UseCase::new(postgres);
    let http_server = HTTPServer::new(cfg.http_server.address, use_case);

    log::info!("start server");

    http_server
        .run()
        .await
        .map_err(|e| format!("failed to run server: {e}"))?;

    Ok(())
}
