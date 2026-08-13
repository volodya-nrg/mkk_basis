mod helpers;

use axum::http::StatusCode;
use fake::faker::internet::en::SafeEmail;
use fake::{Fake, Faker};
use helpers::client::Client;
use helpers::funcs;
use mkk_basis::adapter::db::postgres::Postgres;
use mkk_basis::adapter::db::postgres::tables::tasks::Status as TaskStatus;
use mkk_basis::adapter::jwt::Jwt;
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

static MARKER: OnceCell<String> = OnceCell::const_new();

async fn run_test_server() -> &'static str {
    MARKER.get_or_init(|| async move {
        logger::init("", "", "error", "", true).unwrap();

        let addr_socket = TcpListener::bind("127.0.0.1:0")
            .expect("failed to bind addr")
            .local_addr()
            .expect("failed to local addr");
        let addr_str = addr_socket.to_string();
        let http_addr = format!("http://{}", addr_str);
        let pool = PgPoolOptions::new()
            .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres?search_path=mkk_basis&sslmode=disable")
            .await
            .expect("failed to connect to db");
        let pg_service = Postgres::new(pool);
        let private_key = funcs::generate_private_key_bytes(32);
        let jwt_service = Jwt::new(private_key, 10, 20);
        let use_case = UseCase::new(pg_service, jwt_service);
        let http_server = HTTPServer::new(addr_str.clone(), use_case);

        tokio::spawn(async move {
            log::info!("http-server start on {}", addr_str.clone());
            if let Err(e) = http_server.run().await {
                log::error!("failed to run server: {e}");
            }
        });

        // TODO тут надо проверить готовность сервера
        /*
            // Если сервер возвращает пустое тело, но клиент ожидает данные
            let response = client.post(url)
                .json(&payload)
                .send()
                .await?;

            // Проверьте статус перед чтением тела
            if response.status().is_success() {
                let body = response.text().await?; // Может вызвать IncompleteMessage если тело пустое
            } else {
                // Обработка ошибки
            }
        */

        // let timeout = Duration::from_secs(10);
        // let start = Instant::now();
        // while start.elapsed() < timeout {
        //     if std::net::TcpStream::connect(addr_socket).is_ok() {
        //         return http_addr;
        //     }
        //     sleep(Duration::from_millis(50)).await;
        // }
        // panic!("Server didn't start in time");

        sleep(Duration::from_secs(1)).await;
        http_addr
    }).await
}

#[tokio::test]
async fn check_etc() {
    let cl = Client::new(run_test_server().await.to_string());

    cl.index(|result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());
        assert!(!body_str.is_empty());
    })
    .await
    .healthz(|result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());
        assert!(body_str.is_empty());
    })
    .await
    .page404(|result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .get_file(
        "/robots.txt".to_string(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert!(status_code.is_success());
            assert!(!body_str.is_empty());
        },
    )
    .await
    .get_file(
        "/sitemap.xml".to_string(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert!(status_code.is_success());
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
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_register.email = SafeEmail().fake();
            req_register.password = (1..consts::MIN_PASSWORD_LEN).fake::<String>();
        },
    )
    .await // err: проверка пароля на длину
    .register(
        req_register.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_register.password = Faker.fake::<String>();
        },
    )
    .await // err: проверка паролей на равенство
    .register(
        req_register.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_register.password_confirm = req_register.password.clone();
        },
    )
    .await // ok
    .register(
        req_register.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert!(status_code.is_success());
        },
    )
    .await // err: проверим е-мэйлу не некорректный
    .login(
        req_login.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_login.email = SafeEmail().fake();
        },
    )
    .await // err: проверим что по левому е-мэйлу не находит пользователя
    .login(
        req_login.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_login.email = req_register.email.clone();
            req_login.password = "abc".to_string();
        },
    )
    .await // err: проверим что пароль короткий
    .login(
        req_login.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_login.password = Faker.fake();
        },
    )
    .await // err: проверим что пароль не верный
    .login(
        req_login.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);
        },
    )
    .await // err: 401
    .logout(|result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);

        req_login.password = req_register.password.clone();
    })
    .await // ok
    .login(
        req_login.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert!(status_code.is_success());

            let resp_login: ResponseLogin =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            assert!(!resp_login.access_token.is_empty());
            assert!(!resp_login.refresh_token.is_empty());
        },
    )
    .await // ok
    .logout(|result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // err: 401
    .logout(|result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
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
    let req_login = RequestLogin {
        email: req.email.clone(),
        password: req.password.clone(),
    };

    // проверим на 401
    cl.teams_list(-1, -1, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .teams_create(
        Faker.fake::<RequestTeamCreate>(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await
    .teams_invite(
        Uuid::new_v4(),
        Faker.fake::<RequestTeamInvite>(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await;

    // создадим пользователя и аутентифицируемся
    cl.register(req, |result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        user_id = resp.uuid;
    })
    .await
    .login(req_login, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // err: пользователя нет
    cl.teams_create(
        req_team_create.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, status_code);

            req_team_create.created_by = user_id;
        },
    )
    .await // ок
    .teams_create(
        req_team_create,
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert!(status_code.is_success());

            let resp_team_actual: ResponseTeam =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            assert_eq!(user_id, resp_team_actual.created_by);
            team_id = resp_team_actual.team_id;
        },
    )
    .await
    .teams_list(100, 0, |result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTeamsList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(!resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await
    .teams_list(0, 0, |result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTeamsList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await
    .teams_invite(
        team_id,
        RequestTeamInvite { user_id },
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_success());
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
    let req_login = RequestLogin {
        email: req_register.email.clone(),
        password: req_register.password.clone(),
    };

    // проверим на 401
    cl.tasks_list(-1, -1, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .tasks_create(
        Faker.fake::<RequestTask>(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await
    .tasks_update(
        Uuid::new_v4(),
        Faker.fake::<RequestTask>(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await
    .tasks_history(
        Uuid::new_v4(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await;

    // создадим пользователя, залогинимся и создадим команду
    cl.register(
        req_register,
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert!(status_code.is_success());

            let resp: ResponseUUID =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            user_id = resp.uuid;
            req_team_create.created_by = user_id;
        },
    )
    .await
    .login(req_login, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await
    .teams_create(
        req_team_create,
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert!(status_code.is_success());

            let resp_ream_actual: ResponseTeam =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            team_id = resp_ream_actual.team_id;

            req_task1.created_by = user_id;
            req_task1.team_id = team_id;
            req_task1.assignee_id = None;

            req_task2.created_by = user_id;
            req_task2.team_id = team_id;
            req_task2.assignee_id = Some(user_id);
        },
    )
    .await;

    // ok
    cl.tasks_create(
        req_task1.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert!(status_code.is_success());

            let resp_task: ResponseTask =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            task1_id = resp_task.task_id;
        },
    )
    .await // err: с теми же данными
    .tasks_create(
        req_task1.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, status_code);
        },
    )
    .await
    .tasks_list(100, 0, |result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTasksList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(!resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await
    .tasks_list(0, 0, |result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTasksList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await
    .tasks_update(
        task1_id,
        req_task2.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert!(status_code.is_success());

            let resp: ResponseTask =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            assert_eq!(req_task2.status, resp.status);
            assert_eq!(task1_id, resp.task_id);
        },
    )
    .await
    .tasks_history(task1_id, |result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskHistories =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(resp.items.is_empty());
        // TODO тут надо понять как добавлять историю
    })
    .await;
}
