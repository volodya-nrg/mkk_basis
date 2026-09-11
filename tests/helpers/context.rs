#![allow(dead_code)]

use chrono::{DateTime, Local};
use sqlx::{Pool, Postgres};
use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;
use testcontainers_modules::{
    postgres::Postgres as PostgresContainer,
    testcontainers::ContainerAsync,
    testcontainers::runners::AsyncRunner,
    testcontainers::{ImageExt, core::ContainerPort},
};
use tokio::time::sleep;

use super::{certs, consts, mocks::EmailServiceMock, rand};
use mkk_basis::adapter::db::postgres::transactor::Transactor;
use mkk_basis::{
    adapter::{db::postgres::Postgres as PostgresService, jwt::Jwt as JWTService},
    transport::{self, http_server::HTTPServer},
    usecase::UseCase,
};

pub struct Context {
    pub http_addr: String,
    pub ca: String,
    pub crt: String,
    pub key: String,
    pub time_now: DateTime<Local>,
    pub container: ContainerAsync<PostgresContainer>, // обязательно нужно, чтоб жил, иначе после выходи из ф-ии уничтожается
    pub db: PostgresService,
    pub transactor: Transactor,
}

impl Context {
    pub async fn new() -> Self {
        let _ = Command::new("docker")
            .args(["rm", "-f", consts::CONTAINER_NAME])
            .output();
        let container = PostgresContainer::default()
            .with_container_name(consts::CONTAINER_NAME) // нужно имя, иначе будут плодится
            .with_tag("17.5-alpine3.22")
            .with_mapped_port(consts::DB_PORT, ContainerPort::Tcp(5432))
            .start()
            .await
            .unwrap();
        let pool = Pool::<Postgres>::connect(
            format!(
                "postgres://postgres:postgres@localhost:{}/postgres",
                consts::DB_PORT
            )
            .as_str(),
        )
        .await
        .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let addr_socket = TcpListener::bind(format!("{}:0", certs::LOCALHOST))
            .unwrap()
            .local_addr()
            .unwrap();
        let addr_str = addr_socket.to_string();
        let http_addr = format!("https://{}", addr_str); // явно используем https
        let postgres_service = PostgresService::new();
        let transactor = Transactor::new(pool.clone());
        let use_case = UseCase::new(
            "http://localhost.loc".to_string(),
            postgres_service.clone(),
            JWTService::new(
                rand::private_key(32),
                consts::ACCESS_TOKEN_TTL_SEC,
                consts::REFRESH_TOKEN_TTL_SEC,
            ),
            EmailServiceMock {},
            transactor.clone(),
        );
        let certs = certs::gen_certs().unwrap(); // создадим серты
        // эта штука нужна что определения крипто-провайдера в тесте
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let tls_config = transport::http_server::configure_tls(
            certs.ca_cert.pem().into_bytes(),
            certs.server_cert.pem().into_bytes(),
            certs.server_key.serialize_pem().into_bytes(),
        )
        .unwrap();
        let http_server = HTTPServer::new(addr_str.clone(), use_case, Some(tls_config));

        tokio::spawn(async move { http_server.run().await.unwrap() });
        sleep(Duration::from_secs(1)).await;

        Self {
            http_addr,
            ca: certs.ca_cert.pem(),
            crt: certs.client_cert.pem(),
            key: certs.client_key.serialize_pem(),
            container,
            time_now: Local::now(),
            db: postgres_service,
            transactor,
        }
    }
}
