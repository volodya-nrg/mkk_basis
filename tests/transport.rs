mod helpers;

use axum::http::StatusCode;
use fake::faker::internet::en::SafeEmail;
use fake::{Fake, Faker};
use helpers::client::Client;
use mkk_basis::adapter::db::postgres::Postgres;
use mkk_basis::adapter::logger;
use mkk_basis::consts;
use mkk_basis::transport::http_server::HTTPServer;
use mkk_basis::transport::models::*;
use mkk_basis::usecase::UseCase;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tokio::sync::OnceCell;
use tokio::time::{Duration, sleep};

/*
Проблема: OnceCell создает клиент один раз для всех тестов. Но каждый тест запускается в своем
runtime. Когда первый тест завершается, его runtime может быть уничтожен, но Client (который внутри
содержит свой runtime) остается в статической переменной.

Когда второй тест вызывает get_test_server().await, он получает уже существующий клиент, но его
внутренний runtime уже мог быть уничтожен после завершения первого теста.
*/

static SERVER: OnceCell<String> = OnceCell::const_new();

async fn run_test_server() -> &'static str {
    SERVER.get_or_init(|| async move {
        logger::init("", "", "debug", "").expect("failed to init logger");

        let addr = TcpListener::bind("127.0.0.1:0")
            .expect("failed to bind addr")
            .local_addr()
            .expect("failed to local addr")
            .to_string();
        let http_addr = format!("http://{}", addr);

        tokio::spawn(async move {
            let pool = PgPoolOptions::new()
                .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres?search_path=mkk_basis&sslmode=disable")
                .await
                .expect("failed to connect to db");
            let http_server = HTTPServer::new(addr.clone(), UseCase::new(Postgres::new(pool)));
            log::info!("http-server start on {}", addr.clone());

            if let Err(e) = http_server.run().await {
                log::error!("failed to run server: {e}");
            }
        });

        sleep(Duration::from_secs(1)).await;
        http_addr
    }).await
}

#[tokio::test]
async fn check_etc() {
    let cl = Client::new(run_test_server().await.to_string());

    cl.index(|result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(StatusCode::OK, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .healthz(|result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(StatusCode::OK, status_code);
        assert!(body_str.is_empty());
    })
    .await
    .page404(|result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(StatusCode::NOT_FOUND, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .get_file(
        "/robots.txt".to_string(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::OK, status_code);
            assert!(!body_str.is_empty());
        },
    )
    .await
    .get_file(
        "/sitemap.xml".to_string(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::OK, status_code);
            assert!(!body_str.is_empty());
        },
    )
    .await;
}

#[tokio::test]
async fn check_auth() {
    let cl = Client::new(run_test_server().await.to_string());
    let mut req_register = Faker.fake::<RequestRegister>();
    req_register.email = "abc".to_string();

    // err: проверка е-мэйла на валидность
    cl.register(
        req_register.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);
        },
    )
    .await;
    req_register.email = SafeEmail().fake();

    // err: проверка пароля на длину
    req_register.password = (1..consts::MIN_PASSWORD_LEN).fake::<String>();
    cl.register(
        req_register.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);
        },
    )
    .await;

    // err: проверка паролей на равенство
    req_register.password = Faker.fake::<String>();
    cl.register(
        req_register.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);
        },
    )
    .await;
    req_register.password_confirm = req_register.password.clone();

    // ok
    cl.register(
        req_register.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::OK.as_u16(), status_code);
        },
    )
    .await;

    // err: проверим что по левому е-мэйлу не находит пользователя
    let mut req_login = Faker.fake::<RequestLogin>();
    cl.login(
        req_login.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);
        },
    )
    .await;
    req_login.email = req_register.email.clone();

    // err: проверим что пароль не верный
    cl.login(
        req_login.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);
        },
    )
    .await;
    req_login.password = req_register.password.clone();

    // ok
    cl.login(
        req_login.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(StatusCode::OK.as_u16(), status_code);

            let resp_login: ResponseLogin =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            assert!(!resp_login.access_token.is_empty());
            assert!(!resp_login.refresh_token.is_empty());
        },
    )
    .await;

    // ok
    cl.logout(|result: Result<(u16, String), String>| {
        let (status_code, _) = result.unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(StatusCode::OK.as_u16(), status_code);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn check_transport2() {
    let cl = Client::new(run_test_server().await.to_string());

    cl.index(|result: Result<(u16, String), String>| {
        assert!(result.is_ok());
        let (status_code, body_str) = result.expect("failed to unwrap result");
        assert_eq!(StatusCode::OK, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .page404(|result: Result<(u16, String), String>| {
        assert!(result.is_ok());
        let (status_code, body_str) = result.expect("failed to unwrap result");
        assert_eq!(StatusCode::NOT_FOUND, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .get_file(
        "/robots.txt".to_string(),
        |result: Result<(u16, String), String>| {
            assert!(result.is_ok());
            let (status_code, body_str) = result.expect("failed to unwrap result");
            assert_eq!(StatusCode::OK, status_code);
            assert!(!body_str.is_empty());
        },
    )
    .await
    .get_file(
        "/sitemap.xml".to_string(),
        |result: Result<(u16, String), String>| {
            assert!(result.is_ok());
            let (status_code, body_str) = result.expect("failed to unwrap result");
            assert_eq!(StatusCode::OK, status_code);
            assert!(!body_str.is_empty());
        },
    )
    .await;

    // .register(
    //     Faker.fake::<RequestRegister>(),
    //     |result: Result<(), String>| {
    //         assert!(result.is_ok());
    //     },
    // )
    // .await
    // .login(
    //     Faker.fake::<RequestLogin>(),
    //     |result: Result<ResponseLogin, String>| {
    //         assert!(result.is_ok());
    //         let resp = result.unwrap();
    //         println!("{:?}", resp)
    //     },
    // )
    // .await
    // .logout(|result: Result<reqwest::StatusCode, String>| {
    //     assert!(result.is_ok());
    //     let resp = result.unwrap();
    //     assert_eq!(StatusCode::OK, resp)
    // })
    // .await
    // .teams_list(
    //     100,
    //     0,
    //     "filter".to_string(),
    //     |result: Result<ResponseTeamsList, String>| {
    //         assert!(result.is_ok());
    //         let resp = result.unwrap();
    //         println!("{:?}", resp)
    //     },
    // )
    // .await
    // .teams_create(
    //     Faker.fake::<RequestTeamCreate>(),
    //     |result: Result<ResponseTeam, String>| {
    //         assert!(result.is_ok());
    //         let resp = result.unwrap();
    //         println!("{:?}", resp)
    //     },
    // )
    // .await
    // .teams_invite(
    //     Uuid::new_v4(),
    //     Faker.fake::<RequestTeamInvite>(),
    //     |result: Result<Response, String>| {
    //         assert!(result.is_ok());
    //         let resp = result.unwrap();
    //         assert_eq!(resp.status(), StatusCode::CREATED);
    //     },
    // )
    // .await
    // .tasks_list(
    //     100,
    //     0,
    //     "filter".to_string(),
    //     |result: Result<ResponseTasksList, String>| {
    //         assert!(result.is_ok());
    //         let resp = result.unwrap();
    //         println!("{:?}", resp)
    //     },
    // )
    // .await
    // .tasks_create(
    //     Faker.fake::<RequestTask>(),
    //     |result: Result<ResponseTask, String>| {
    //         assert!(result.is_ok());
    //         let resp = result.unwrap();
    //         println!("{:?}", resp)
    //     },
    // )
    // .await
    // .tasks_update(
    //     Uuid::new_v4(),
    //     Faker.fake::<RequestTask>(),
    //     |result: Result<ResponseTask, String>| {
    //         assert!(result.is_ok());
    //         let resp = result.unwrap();
    //         println!("{:?}", resp)
    //     },
    // )
    // .await
    // .tasks_history(
    //     Uuid::new_v4(),
    //     |result: Result<ResponseTaskHistories, String>| {
    //         assert!(result.is_ok());
    //         let resp = result.unwrap();
    //         assert!(resp.items.len() > 0)
    //     },
    // )
    // .await;
}
