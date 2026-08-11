mod helpers;

use axum::http::StatusCode;
use defer::defer;
use fake::faker::internet::en::SafeEmail;
use fake::{Fake, Faker};
use helpers::client::Client;
use mkk_basis::adapter::db::postgres::Postgres;
use mkk_basis::adapter::db::postgres::tables::tasks::Status as TaskStatus;
use mkk_basis::adapter::logger;
use mkk_basis::consts;
use mkk_basis::transport::http_server::HTTPServer;
use mkk_basis::transport::models::*;
use mkk_basis::usecase::UseCase;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tokio::sync::OnceCell;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

/*
Проблема: OnceCell создает клиент один раз для всех тестов. Но каждый тест запускается в своем
runtime. Когда первый тест завершается, его runtime может быть уничтожен, но Client (который внутри
содержит свой runtime) остается в статической переменной.

Когда второй тест вызывает get_test_server().await, он получает уже существующий клиент, но его
внутренний runtime уже мог быть уничтожен после завершения первого теста.
*/

static MARKER: OnceCell<String> = OnceCell::const_new();

async fn run_test_server() -> &'static str {
    MARKER.get_or_init(|| async move {
        logger::init("test_transport_service", "v0.0.1", "debug", "", true).expect("failed to init logger");

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
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .healthz(|result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK, status_code);
        assert!(body_str.is_empty());
    })
    .await
    .page404(|result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .get_file(
        "/robots.txt".to_string(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::OK, status_code);
            assert!(!body_str.is_empty());
        },
    )
    .await
    .get_file(
        "/sitemap.xml".to_string(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap();
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
    let mut req_login = Faker.fake::<RequestLogin>();
    req_login.email = "abc".to_string();

    // err: проверка е-мэйла на валидность
    cl.register(
        req_register.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);

            // err: проверка пароля на длину
            req_register.email = SafeEmail().fake();
            req_register.password = (1..consts::MIN_PASSWORD_LEN).fake::<String>();
        },
    )
    .await
    .register(
        req_register.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);

            // err: проверка паролей на равенство
            req_register.password = Faker.fake::<String>();
        },
    )
    .await
    .register(
        req_register.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);

            // ok
            req_register.password_confirm = req_register.password.clone();
        },
    )
    .await
    .register(
        req_register.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::OK.as_u16(), status_code);
        },
    )
    .await // err: проверим е-мэйлу не некорректный
    .login(
        req_login.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);

            // err: проверим что по левому е-мэйлу не находит пользователя
            req_login.email = SafeEmail().fake();
        },
    )
    .await
    .login(
        req_login.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);

            // err: проверим что пароль короткий
            req_login.email = req_register.email.clone();
            req_login.password = "abc".to_string();
        },
    )
    .await
    .login(
        req_login.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);

            // err: проверим что пароль не верный
            req_login.password = Faker.fake();
        },
    )
    .await
    .login(
        req_login.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST.as_u16(), status_code);

            // ok
            req_login.password = req_register.password.clone();
        },
    )
    .await
    .login(
        req_login.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::OK.as_u16(), status_code);

            let resp_login: ResponseLogin =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            assert!(!resp_login.access_token.is_empty());
            assert!(!resp_login.refresh_token.is_empty());
        },
    )
    .await
    .logout(|result: Result<(u16, String), String>| {
        let (status_code, _) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);
    })
    .await;
}

#[tokio::test]
async fn check_teams() {
    let cl = Client::new(run_test_server().await.to_string());
    let mut user_id = Uuid::nil();
    let mut req_team_create = Faker.fake::<RequestTeamCreate>();
    let mut req = Faker.fake::<RequestRegister>();
    req.password_confirm = req.password.clone();
    let mut team_id = Uuid::nil();

    // создадим пользователя
    cl.register(req, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        user_id = resp.uuid;
    })
    .await;
    defer!({
        // TODO удалить пользователя
    });

    // err: пользователя нет
    cl.teams_create(
        req_team_create.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), status_code);

            req_team_create.created_by = user_id;
        },
    )
    .await // ок
    .teams_create(req_team_create, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp_team_actual: ResponseTeam =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert_eq!(user_id, resp_team_actual.created_by);
        team_id = resp_team_actual.team_id;
    })
    .await
    .teams_list(100, 0, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp: ResponseTeamsList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(!resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await
    .teams_list(0, 0, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp: ResponseTeamsList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await
    .teams_invite(
        team_id,
        RequestTeamInvite { user_id },
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::OK.as_u16(), status_code);
        },
    )
    .await;
}

#[tokio::test]
async fn check_tasks() {
    let cl = Client::new(run_test_server().await.to_string());
    let mut user_id = Uuid::nil();
    let mut team_id = Uuid::nil();
    let mut task1_id = Uuid::nil();
    let mut req_register = Faker.fake::<RequestRegister>();
    req_register.password_confirm = req_register.password.clone();
    let mut req_team_create = Faker.fake::<RequestTeamCreate>();
    let mut req_task1 = Faker.fake::<RequestTask>();
    req_task1.status = TaskStatus::Start.to_string();
    let mut req_task2 = Faker.fake::<RequestTask>();
    req_task2.status = TaskStatus::Cancelled.to_string();

    // создадим пользователя и команду
    cl.register(req_register, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        user_id = resp.uuid;
        req_team_create.created_by = user_id;
    })
    .await
    .teams_create(req_team_create, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp_ream_actual: ResponseTeam =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        team_id = resp_ream_actual.team_id;

        req_task1.created_by = user_id;
        req_task1.team_id = team_id;
        req_task1.assignee_id = None;

        req_task2.created_by = user_id;
        req_task2.team_id = team_id;
        req_task2.assignee_id = Some(user_id);
    })
    .await;

    // ok
    cl.tasks_create(
        req_task1.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::OK.as_u16(), status_code);

            let resp_task: ResponseTask =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            task1_id = resp_task.task_id;
        },
    )
    .await // err: с теми же данными
    .tasks_create(
        req_task1.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, _) = result.unwrap();
            assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), status_code);
        },
    )
    .await
    .tasks_list(100, 0, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp: ResponseTasksList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(!resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await
    .tasks_list(0, 0, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp: ResponseTasksList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await
    .tasks_update(
        task1_id,
        req_task2.clone(),
        |result: Result<(u16, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert_eq!(StatusCode::OK.as_u16(), status_code);

            let resp: ResponseTask =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            assert_eq!(req_task2.status, resp.status);
            assert_eq!(task1_id, resp.task_id);
        },
    )
    .await
    .tasks_history(task1_id, |result: Result<(u16, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::OK.as_u16(), status_code);

        let resp: ResponseTaskHistories =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(resp.items.is_empty());
        // TODO тут надо понять как добавлять историю
    })
    .await;
}
