use super::common::client;
use super::common::helpers;
use fake::{Fake, Faker};
use mkk_basis::adapter::db::postgres::Postgres;
use mkk_basis::adapter::logger;
use mkk_basis::transport::http_server::HTTPServer;
use mkk_basis::transport::models::{RequestRegister, ResponseRegister};
use mkk_basis::usecase::UseCase;
use sqlx::postgres::PgPoolOptions;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn check_transport() {
    let addr = format!("localhost:{}", helpers::gen_rand_port());
    let http_addr = format!("http://{}", addr);
    let cl = client::Client::new(http_addr);

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

    let req_register: RequestRegister = Faker.fake();
    cl.register(req_register, |result: Result<ResponseRegister, String>| {
        assert!(result.is_ok());
        let resp = result.unwrap();
        println!("{:?}", resp)
    })
    .await;
}
