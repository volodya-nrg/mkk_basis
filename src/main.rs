use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::fs;
use std::process;
use std::time::Duration;

use mkk_basis::adapter::{
    config::Config,
    db::postgres::{
        Postgres as PostgresService,
        transactor::{IsolationLevel, Transactor},
    },
    email::Email as EmailService,
    jwt::Jwt as JWTService,
    logger,
};
use mkk_basis::consts;
use mkk_basis::transport;
use mkk_basis::transport::http_server::HTTPServer;
use mkk_basis::usecase::UseCase;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "./data/config.yaml")]
    config: String,
}

/*
async-await:
    асинхронный код
    не блокирует текущий поток, который может взять после др. задачу на выполнение
    возвращает обещание (future) результата
    await - приостанови эту задачу пока future не будет готов и отдай поток другим задачам. Выполнение может остановится и продолжить позже.
    небходим executor который продвигает эту задачу. Он делает poll, пока не получит результат.
runtime:
    многопотомный runtime с ланировщиком, I/O-драйвером и таймером
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {...});
#[tokio::main]:
    tokio::runtime::Builder::new_multi_thread()
        .enable_all() // включает I/O-драйвер и таймеры
        .build() // строит runtime
        .unwrap()
        .block_on(async {
            println!("Hello world"); // запускает future
        })
    По умолчанию используется многопоточный runtime (multi_thread) с числом воркеров, равным количеству ядер CPU.
*/

#[tokio::main] // атрибут-макрос, создает tokio-runtime
async fn main() {
    if let Err(e) = run(Args::parse().config).await {
        log::error!("failed to run app: {e}");
        process::exit(1);
    }
}

async fn run(config_filepath: String) -> Result<(), String> {
    // возвращаешься тип Unit ("()")
    let cfg = Config::new(&config_filepath).map_err(|e| {
        let str = format!("failed to create new config: {e}");
        eprint!("{}", str);
        str
    })?;

    logger::init(
        cfg.service_name.clone(),
        cfg.version.clone(),
        &cfg.log.level,
        cfg.log.filepath,
        false,
    )
    .map_err(|e| format!("failed to init logger: {e}"))?;

    let private_key_bytes =
        fs::read(cfg.private_key_path).map_err(|e| format!("failed to read private key: {e}"))?;
    let tls_config_for_server = if cfg.http_server.tls.is_use {
        let ca_bytes = fs::read(cfg.http_server.tls.ca_filepath)
            .map_err(|e| format!("failed to read ca file: {e}"))?;
        let crt_bytes = fs::read(cfg.http_server.tls.crt_filepath)
            .map_err(|e| format!("failed to read crt file: {e}"))?;
        let key_bytes = fs::read(cfg.http_server.tls.key_filepath)
            .map_err(|e| format!("failed to read key file: {e}"))?;
        let tls_config = transport::http_server::configure_tls(ca_bytes, crt_bytes, key_bytes)
            .map_err(|e| format!("failed to configure tls: {e}"))?;
        Some(tls_config)
    } else {
        None
    };

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::new(3, 0))
        .connect(&cfg.postgres.dsn)
        .await
        .map_err(|e| format!("failed to connect on DB: {e}"))?;
    let transactor = Transactor::new(pool, IsolationLevel::ReadCommitted);
    let http_server = HTTPServer::new(
        cfg.http_server.address.clone(),
        UseCase::new(
            cfg.addr,
            PostgresService::new(),
            JWTService::new(
                private_key_bytes,
                consts::ACCESS_TOKEN_TTL_SEC,
                consts::REFRESH_TOKEN_TTL_SEC,
            ),
            EmailService::new(
                &cfg.email.host,
                &cfg.email.login,
                &cfg.email.pass,
                &cfg.email.from_email,
                &cfg.email.from_name,
                Duration::from_secs(3),
            ),
            transactor,
        ),
        tls_config_for_server.clone(),
    );

    http_server
        .run()
        .await
        .map_err(|e| format!("failed to run server: {e}"))
}
