mod helpers;

use axum::http::StatusCode;
use helpers::client::Client;
use helpers::rand;
use mkk_basis::adapter::db::postgres::Postgres;
use mkk_basis::adapter::jwt::Jwt;
use mkk_basis::adapter::logger;
use mkk_basis::transport::http_server::HTTPServer;
use mkk_basis::transport::models::*;
use mkk_basis::usecase::UseCase;
use mkk_basis::{consts, transport};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tokio::sync::OnceCell;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

struct ClientData {
    http_addr: String,
    ca: String,
    crt: String,
    key: String,
}

const DSN: &str =
    "postgres://postgres:postgres@127.0.0.1:5432/postgres?options=-c%20search_path%3Dmkk_basis";
static MARKER: OnceCell<ClientData> = OnceCell::const_new();

async fn run_test_server() -> &'static ClientData {
    MARKER
        .get_or_init(|| async move {
            logger::init("", "", "", "", true).unwrap();

            let addr_socket = TcpListener::bind(format!("{}:0", helpers::certs::LOCALHOST))
                .expect("failed to bind addr")
                .local_addr()
                .expect("failed to local addr");
            let addr_str = addr_socket.to_string();
            let http_addr = format!("https://{}", addr_str); // явно используем https
            let pool = PgPoolOptions::new()
                .connect(DSN)
                .await
                .expect("failed to connect to db");
            let pg_service = Postgres::new(pool);
            let private_key = rand::private_key(32);
            let jwt_service = Jwt::new(private_key, 10, 20);
            let use_case = UseCase::new(pg_service, jwt_service);
            let certs = helpers::certs::gen_certs().unwrap(); // создадим серты
            let tls_config = transport::http_server::configure_tls(
                certs.ca_cert.pem().into_bytes(),
                certs.server_cert.pem().into_bytes(),
                certs.server_key.serialize_pem().into_bytes(),
            )
            .expect("failed to configure tls");
            let http_server = HTTPServer::new(addr_str.clone(), use_case, Some(tls_config));

            tokio::spawn(async move {
                if let Err(e) = http_server.run().await {
                    log::error!("failed to run server: {e}");
                }
            });

            sleep(Duration::from_secs(1)).await;

            ClientData {
                http_addr,
                ca: certs.ca_cert.pem(),
                crt: certs.client_cert.pem(),
                key: certs.client_key.serialize_pem(),
            }
        })
        .await
}

#[tokio::test]
async fn check_etc() {
    let data = run_test_server().await;
    let cl = Client::new(
        data.http_addr.to_string(),
        data.ca.to_string(),
        data.crt.to_string(),
        data.key.to_string(),
    );

    cl.index(|result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());
        assert!(!body_str.is_empty());
    })
    .await
    .health(|result: Result<(StatusCode, String), String>| {
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
    let data = run_test_server().await;
    let cl = Client::new(
        data.http_addr.to_string(),
        data.ca.to_string(),
        data.crt.to_string(),
        data.key.to_string(),
    );
    let wrong_email = "abc".to_string();
    let mut req_register = rand::request_register();
    let mut req_login = rand::request_login();

    req_register.email = wrong_email.clone();
    req_login.email = wrong_email.clone();

    // err: проверка е-мэйла на валидность
    cl.register(
        req_register.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_register.email = rand::email();
            req_register.password = rand::str_limit(consts::MIN_PASSWORD_LEN - 1);
        },
    )
    .await // err: проверка пароля на длину
    .register(
        req_register.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_register.password = rand::str();
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

            req_login.email = rand::email();
        },
    )
    .await // err: проверим что по левому е-мэйлу не находит пользователя
    .login(
        req_login.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_login.email = req_register.email.clone();
            req_login.password = rand::str_limit(consts::MIN_PASSWORD_LEN - 1);
        },
    )
    .await // err: проверим что пароль короткий
    .login(
        req_login.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);

            req_login.password = rand::str();
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
    let data = run_test_server().await;
    let cl = Client::new(
        data.http_addr.to_string(),
        data.ca.to_string(),
        data.crt.to_string(),
        data.key.to_string(),
    );

    let mut user_id = Uuid::nil();
    let mut req_team_create = rand::request_team_create();
    let req_register = rand::request_register();
    let mut team_id = Uuid::nil();
    let req_login = RequestLogin {
        email: req_register.email.clone(),
        password: req_register.password.clone(),
    };

    // проверим на 401
    cl.teams_list(-1, -1, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .teams_create(
        rand::request_team_create(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await
    .teams_invite(
        Uuid::new_v4(),
        rand::request_team_invite(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await;

    // создадим пользователя и аутентифицируемся
    cl.register(
        req_register,
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert!(status_code.is_success());

            let resp: ResponseUUID =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            user_id = resp.uuid;
        },
    )
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
    let data = run_test_server().await;
    let cl = Client::new(
        data.http_addr.to_string(),
        data.ca.to_string(),
        data.crt.to_string(),
        data.key.to_string(),
    );

    let mut user_id = Uuid::nil();
    let mut team_id = Uuid::nil();
    let mut task1_id = Uuid::nil();
    let mut req_register = rand::request_register();
    let mut req_team_create = rand::request_team_create();
    let mut req_task1 = rand::request_task();
    let mut req_task2 = rand::request_task();
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
        rand::request_task(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await
    .tasks_update(
        Uuid::new_v4(),
        rand::request_task(),
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

#[tokio::test]
async fn check_users() {
    let data = run_test_server().await;
    let cl = Client::new(
        data.http_addr.to_string(),
        data.ca.to_string(),
        data.crt.to_string(),
        data.key.to_string(),
    );

    let mut owner_id = Uuid::nil();
    let mut user_id = Uuid::nil();
    let req_register = rand::request_register();
    let req_user1 = rand::request_user();
    let req_user2 = rand::request_user();
    let req_login = RequestLogin {
        email: req_register.email.clone(),
        password: req_register.password.clone(),
    };
    let mut saved_resp_user: ResponseUser = ResponseUser {
        user_id,
        name: None,
        email: "".to_string(),
        email_is_confirmed: false,
        avatar: None,
        created_at: Default::default(),
        updated_at: Default::default(),
    };

    // проверим на 401
    cl.users_list(-1, -1, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .users_create(
        rand::request_user(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await
    .users_update(
        Uuid::new_v4(),
        rand::request_user(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await
    .users_delete(
        Uuid::new_v4(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert_eq!(StatusCode::UNAUTHORIZED, status_code);
        },
    )
    .await;

    // создадим пользователя, залогинимся и создадим др. пользователя
    cl.register(
        req_register,
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap_or_else(|e| panic!("{:?}", e));
            assert!(status_code.is_success());

            let resp: ResponseUUID =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            owner_id = resp.uuid;
        },
    )
    .await
    .login(req_login, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // err: пользователя нет
    cl.users_one(
        Uuid::new_v4(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::NOT_FOUND, status_code);
        },
    )
    .await // err: такого пользователя нет
    .users_delete(
        Uuid::new_v4(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_server_error()); // TODO тут под вопросом 404 надо присылать или 500
        },
    )
    .await // err: такого пользователя нет
    .users_update(
        Uuid::new_v4(),
        rand::request_user(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_server_error()); // TODO тут под вопросом 404 надо присылать или 500
        },
    )
    .await // ok: создадим успешно
    .users_create(
        req_user1.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert!(status_code.is_success());

            let resp_user_actual: ResponseUser =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            assert_eq!(req_user1.email, resp_user_actual.email);
            assert_eq!(req_user1.name, resp_user_actual.name);
            assert_eq!(
                req_user1.email_is_confirmed,
                resp_user_actual.email_is_confirmed
            );

            user_id = resp_user_actual.user_id;
            saved_resp_user = resp_user_actual;
        },
    )
    .await // ok: получим запись
    .users_one(user_id, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // ok: обновим успешно
    .users_update(
        user_id,
        req_user2.clone(),
        |result: Result<(StatusCode, String), String>| {
            let (status_code, body_str) = result.unwrap();
            assert!(status_code.is_success());

            let resp_user_actual: ResponseUser =
                serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
            assert_ne!(saved_resp_user, resp_user_actual);
        },
    )
    .await // ok: посмотрим что люди есть
    .users_list(0, 0, |result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let list: ResponseUsersList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert_eq!(list.items.len(), 0);
        assert!(list.total > 0);
    })
    .await // ok: найдем нужное и сравним
    .users_list(-1, -1, |result: Result<(StatusCode, String), String>| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUsersList =
            serde_json::from_str(body_str.as_str()).expect("failed to parse str to json");
        assert!(resp.items.len() > 0);
        assert!(resp.total > 0);

        let opt = resp.items.iter().find(|item| item.user_id == user_id);
        assert!(opt.is_some());

        let founded_user = opt.unwrap();
        assert_eq!(req_user2.email, founded_user.email);
        assert_eq!(req_user2.name, founded_user.name);
        assert_eq!(
            req_user2.email_is_confirmed,
            founded_user.email_is_confirmed
        );
    })
    .await // ок: удалим успешно
    .users_delete(user_id, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // ok: пользователя не должно быть
    .users_one(user_id, |result: Result<(StatusCode, String), String>| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
    })
    .await;
}
