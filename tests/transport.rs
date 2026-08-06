mod helpers;

use axum::http::StatusCode;
use fake::{Fake, Faker};
use mkk_basis::adapter::db::postgres::Postgres;
use mkk_basis::adapter::logger;
use mkk_basis::transport::http_server::HTTPServer;
use mkk_basis::transport::models::*;
use mkk_basis::usecase::UseCase;
use reqwest::Response;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

#[tokio::test]
async fn check_transport() {
    logger::init("", "", "debug", "").unwrap();

    let addr = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .to_string();
    let http_addr = format!("http://{}", addr);
    let cl = helpers::client::Client::new(http_addr);

    tokio::spawn(async move {
        let pool = PgPoolOptions::new()
            .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres?search_path=mkk_basis&sslmode=disable")
            .await
            .unwrap();
        let http_server = HTTPServer::new(addr.clone(), UseCase::new(Postgres::new(pool)));
        log::info!("http-server start on {}", addr);
        http_server.run().await.unwrap()
    });

    sleep(Duration::from_secs(1)).await;

    cl.index(|result: Result<(u16, String), String>| {
        assert!(result.is_ok());
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .page404(|result: Result<(u16, String), String>| {
        assert!(result.is_ok());
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .get_file(
        "/robots.txt".to_string(),
        |result: Result<(u16, String), String>| {
            assert!(result.is_ok());
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::OK, status_code);
            assert!(!body_str.is_empty());
        },
    )
    .await
    .get_file(
        "/sitemap.xml".to_string(),
        |result: Result<(u16, String), String>| {
            assert!(result.is_ok());
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::OK, status_code);
            assert!(!body_str.is_empty());
        },
    )
    .await
    .register(
        Faker.fake::<RequestRegister>(),
        |result: Result<(), String>| {
            assert!(result.is_ok());
        },
    )
    .await
    .login(
        Faker.fake::<RequestLogin>(),
        |result: Result<ResponseLogin, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            println!("{:?}", resp)
        },
    )
    .await
    .logout(|result: Result<reqwest::StatusCode, String>| {
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(StatusCode::OK, resp)
    })
    .await
    .teams_list(
        100,
        0,
        "filter".to_string(),
        |result: Result<ResponseTeamsList, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            println!("{:?}", resp)
        },
    )
    .await
    .teams_create(
        Faker.fake::<RequestTeamCreate>(),
        |result: Result<ResponseTeam, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            println!("{:?}", resp)
        },
    )
    .await
    .teams_invite(
        Uuid::new_v4(),
        Faker.fake::<RequestTeamInvite>(),
        |result: Result<Response, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);
        },
    )
    .await
    .tasks_list(
        100,
        0,
        "filter".to_string(),
        |result: Result<ResponseTasksList, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            println!("{:?}", resp)
        },
    )
    .await
    .tasks_create(
        Faker.fake::<RequestTask>(),
        |result: Result<ResponseTask, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            println!("{:?}", resp)
        },
    )
    .await
    .tasks_update(
        Uuid::new_v4(),
        Faker.fake::<RequestTask>(),
        |result: Result<ResponseTask, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            println!("{:?}", resp)
        },
    )
    .await
    .tasks_history(
        Uuid::new_v4(),
        |result: Result<ResponseTaskHistories, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            assert!(resp.items.len() > 0)
        },
    )
    .await;
}
