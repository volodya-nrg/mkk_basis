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
use tokio::time::{Duration, sleep};
use uuid::Uuid;

#[tokio::test]
async fn check_transport() {
    let addr = format!("localhost:{}", helpers::funcs::gen_rand_port());
    let http_addr = format!("http://{}", addr);
    let cl = helpers::client::Client::new(http_addr);

    logger::init("", "", "debug", "").unwrap();

    tokio::spawn(async move {
        let pool = PgPoolOptions::new()
            .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres?search_path=mkk_basis&sslmode=disable")
            .await
            .unwrap();
        HTTPServer::run(&addr, UseCase::new(Postgres::new(&pool)))
            .await
            .unwrap()
    });

    sleep(Duration::from_secs(1)).await;

    cl.register(
        Faker.fake::<RequestRegister>(),
        |result: Result<ResponseRegister, String>| {
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
        Faker.fake::<RequestTaskCreate>(),
        |result: Result<ResponseTask, String>| {
            assert!(result.is_ok());
            let resp = result.unwrap();
            println!("{:?}", resp)
        },
    )
    .await;
}
