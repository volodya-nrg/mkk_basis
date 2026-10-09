#![allow(dead_code)]

use chrono::{DateTime, Local};
use sqlx::{Pool, Postgres as SQLXPostgres};
use std::net::TcpListener;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::{
    postgres::Postgres as PostgresContainer,
    testcontainers::ContainerAsync,
    testcontainers::runners::AsyncRunner,
    testcontainers::{ImageExt, core::ContainerPort},
};
use tokio::time::sleep;

use mkk_basis::{
    adapter::{
        db::postgres::connection::Postgres, db::storage::IsolationLevel, jwt::Jwt as JWTService,
        logger,
    },
    transport::{self, http_server::HTTPServer},
    usecase::index::UseCase,
};

use super::{certs, consts, mocks::EmailServiceMock, rand};
use crate::common::mocks::ConfirmationCodeStoreMock;

pub struct Context {
    pub http_addr: String,
    pub ca: String,
    pub crt: String,
    pub key: String,
    pub time_now: DateTime<Local>,
    pub container: ContainerAsync<PostgresContainer>, // обязательно нужно, чтоб жил, иначе после выходи из ф-ии уничтожается
    pub db: Arc<Postgres>,
    pub code_store: Arc<ConfirmationCodeStoreMock>,
}

impl Context {
    pub async fn new() -> Self {
        logger::init(String::new(), String::new(), "info", None, true).unwrap();

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
        let pool = Pool::<SQLXPostgres>::connect(
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
        let arc_storage_service =
            Arc::new(Postgres::new(pool.clone(), IsolationLevel::Serializable));
        let arc_coder = Arc::new(ConfirmationCodeStoreMock::new());
        let use_case = UseCase::new(
            "http://localhost.loc".to_string(),
            arc_storage_service.clone(),
            JWTService::new(
                rand::private_key(32),
                consts::ACCESS_TOKEN_TTL_SEC,
                consts::REFRESH_TOKEN_TTL_SEC,
            ),
            Arc::new(EmailServiceMock {}),
            arc_coder.clone(),
        );
        let certs = certs::gen_certs().unwrap(); // создадим серты
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
            db: arc_storage_service,
            code_store: arc_coder,
        }
    }
}
