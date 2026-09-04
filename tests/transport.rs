mod helpers;

use axum::http::StatusCode;
use ctor::ctor;
use std::time::Duration;
use tokio::sync::OnceCell;
use tokio::time::sleep;
use uuid::Uuid;

use mkk_basis::{
    adapter::{db::postgres::tables::users::Role as UsersRole, helpers as HelpersService, logger},
    consts::MIN_PASSWORD_LEN,
    transport::models::{
        RequestLogin, RequestTaskData, RequestTeamInvite, RequestUserUpdate, ResponseLogin,
        ResponseRefreshToken, ResponseTask, ResponseTaskComment, ResponseTaskCommentsList,
        ResponseTaskHistories, ResponseTasksList, ResponseTeam, ResponseTeamsList, ResponseUUID,
        ResponseUser, ResponseUsersList,
    },
};

use helpers::{client::Client, consts, context::Context, rand};

#[ctor(unsafe)]
fn init() {
    logger::init(String::new(), String::new(), String::new(), None, true).unwrap()
}

static CONTEXT: OnceCell<Context> = OnceCell::const_new();

async fn get_context() -> &'static Context {
    CONTEXT.get_or_init(|| async { Context::new().await }).await
}

#[tokio::test]
async fn check_etc() {
    let ctx = get_context().await;
    let mut cl = Client::new(
        ctx.http_addr.to_string(),
        ctx.ca.to_string(),
        ctx.crt.to_string(),
        ctx.key.to_string(),
        &ctx.db,
    );

    cl.index(|result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());
        assert!(!body_str.is_empty());
    })
    .await
    .health(|result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());
        assert!(body_str.is_empty());
    })
    .await
    .page404(|result| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
        assert!(!body_str.is_empty());
    })
    .await
    .get_file("/robots.txt".to_string(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());
        assert!(!body_str.is_empty());
    })
    .await
    .get_file("/sitemap.xml".to_string(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());
        assert!(!body_str.is_empty());
    })
    .await;
}

#[tokio::test]
async fn check_auth() {
    let ctx = get_context().await;
    let mut cl = Client::new(
        ctx.http_addr.to_string(),
        ctx.ca.to_string(),
        ctx.crt.to_string(),
        ctx.key.to_string(),
        &ctx.db,
    );

    let wrong_email = "abc".to_string();
    let req_register1 = rand::request_register();
    let mut req_register2 = rand::request_register();
    let mut req_login = rand::request_login();

    // предварительно зарегистрируем одного пользователя
    cl.register(req_register1.clone(), true, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::OK, status_code);
    })
    .await;

    req_register2.email = wrong_email.clone();

    // err: проверка е-мэйла на валидность
    cl.register(req_register2.clone(), false, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);

        req_register2.email = rand::email();
        req_register2.password = HelpersService::rand_str_limit(MIN_PASSWORD_LEN - 1);
    })
    .await // err: проверка пароля на длину
    .register(req_register2.clone(), false, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);

        req_register2.password = rand::str();
    })
    .await // err: проверка паролей на равенство
    .register(req_register2.clone(), false, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);

        req_register2.password_confirm = req_register2.password.clone();
        req_register2.agreement = false;
    })
    .await // err: не принято условия оферты
    .register(req_register2.clone(), false, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);

        req_register2.agreement = true;
        req_register2.privacy_policy = false;
    })
    .await // err: не принято политику конфиденциальности
    .register(req_register2.clone(), false, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);

        req_register2.privacy_policy = true;
    })
    .await // ok
    .register(req_register2.clone(), false, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // err: - не хватает е-мэйла
    cl.register_confirm(None, None, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_client_error());
    })
    .await // err - не хватает code
    .register_confirm(Some(rand::email()), None, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_client_error());
    })
    .await // err - не валидный е-мэйл
    .register_confirm(Some(rand::str()), Some(rand::str()), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_client_error());
    })
    .await // err - пользователь не найден
    .register_confirm(Some(rand::email()), Some(rand::str()), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_client_error());
    })
    .await // запросим проверенного пользователя
    .register_confirm(Some(req_register1.email), Some(rand::str()), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_client_error());
    })
    .await // не верный код
    .register_confirm(
        Some(req_register2.email.clone()),
        Some(rand::str()),
        |result| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_client_error());
        },
    )
    .await;

    // err: попробуем залогинится ("е-мэйл необходимо верифицировать")
    cl.login(
        RequestLogin {
            email: req_register2.email.clone(),
            password: req_register2.password.clone(),
        },
        |result| {
            let (status_code, _body_str) = result.unwrap();
            assert_eq!(StatusCode::BAD_REQUEST, status_code);
        },
    )
    .await;

    // достанем явно код
    let email_code = cl
        .pg_service
        .tbl_users
        .by_email(req_register2.email.clone())
        .await
        .unwrap()
        .email_code
        .unwrap();

    // ok
    cl.register_confirm(
        Some(req_register2.email.clone()),
        Some(email_code),
        |result| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_success());
        },
    )
    .await;

    req_login.email = wrong_email.clone();

    // err: проверим е-мэйлу не некорректный
    cl.login(req_login.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);

        req_login.email = rand::email();
    })
    .await // err: проверим что по левому е-мэйлу не находит пользователя
    .login(req_login.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);

        req_login.email = req_register2.email.clone();
        req_login.password = HelpersService::rand_str_limit(MIN_PASSWORD_LEN - 1);
    })
    .await // err: проверим что пароль короткий
    .login(req_login.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);

        req_login.password = rand::str();
    })
    .await // err: проверим что пароль не верный
    .login(req_login.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, status_code);
    })
    .await // err: 401
    .logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);

        req_login.password = req_register2.password.clone();
    })
    .await // ok
    .login(req_login.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_login: ResponseLogin =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(!resp_login.access_token.is_empty());
        assert!(!resp_login.refresh_token.is_empty());
    })
    .await;

    // err - не верный токен
    let mut req_refresh_token = rand::request_refresh_token();
    cl.refresh_tokens(req_refresh_token.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_client_error());
    })
    .await;

    // err - подставим явно access-token
    req_refresh_token.token = cl.access_token.clone();
    cl.refresh_tokens(req_refresh_token.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_client_error());
    })
    .await;

    // ok
    req_refresh_token.token = cl.refresh_token.clone();
    cl.refresh_tokens(req_refresh_token.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_login: ResponseRefreshToken =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(!resp_login.access_token.is_empty());
        assert!(!resp_login.refresh_token.is_empty());
    })
    .await;

    // ok
    cl.logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // err: 401
    .logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await;

    // залогинимся и подождем пока токен обновления не протухнет
    cl.login(req_login.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    sleep(Duration::from_secs(
        consts::REFRESH_TOKEN_TTL_SEC.cast_unsigned() + 1,
    ))
    .await;

    req_refresh_token.token = cl.refresh_token.clone();
    cl.refresh_tokens(req_refresh_token.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_client_error());
    })
    .await;
}

#[tokio::test]
async fn check_teams() {
    let ctx = get_context().await;
    let mut cl = Client::new(
        ctx.http_addr.to_string(),
        ctx.ca.to_string(),
        ctx.crt.to_string(),
        ctx.key.to_string(),
        &ctx.db,
    );

    let mut user_id = Uuid::nil();
    let mut req_team = rand::request_team();
    let req_register = rand::request_register();
    let mut team_id = Uuid::nil();
    let req_login = RequestLogin {
        email: req_register.email.clone(),
        password: req_register.password.clone(),
    };

    // проверим на 401
    cl.teams_list(-1, -1, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .teams_one(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .teams_create(rand::request_team(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .teams_update(Uuid::new_v4(), rand::request_team(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .teams_delete(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .teams_invite(Uuid::new_v4(), rand::request_team_invite(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await;

    // создадим пользователя и аутентифицируемся
    cl.register(req_register, true, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        user_id = resp.uuid;
    })
    .await
    .login(req_login, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // ok. Создатель user_id, т.к. он создал, он является участником группы.
    cl.teams_create(req_team.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_team_actual: ResponseTeam =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(user_id, resp_team_actual.created_by);
        team_id = resp_team_actual.team_id;
    })
    .await // err - нельзя создать дубликат
    .teams_create(req_team.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_server_error());
    })
    .await // ok
    .teams_list(100, 0, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTeamsList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(!resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await // ok
    .teams_list(0, 0, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTeamsList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(resp.items.is_empty());
        assert!(resp.total > 0);
    })
    .await // ok
    .teams_one(team_id, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTeam =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(team_id, resp.team_id)
    })
    .await // err
    .teams_one(Uuid::new_v4(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);

        req_team.name = rand::str()
    })
    .await // ok - обновим имя и проверим его
    .teams_update(team_id, req_team.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTeam =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(req_team.name, resp.name)
    })
    .await // ok
    .teams_delete(team_id, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await
    .teams_one(team_id, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
    })
    .await;

    // проверим приглашения
    let mut admin_id = Uuid::nil();
    let mut owner_id = Uuid::nil();
    let mut other_id = Uuid::nil();
    let req_register_admin = rand::request_register();
    let req_register_owner = rand::request_register();
    let req_register_other = rand::request_register();

    // создадим admin, owner, other
    cl.register(req_register_admin.clone(), true, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        admin_id = resp.uuid;
    })
    .await
    .register(req_register_owner.clone(), true, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        owner_id = resp.uuid;
    })
    .await
    .register(req_register_other.clone(), true, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        other_id = resp.uuid;
    })
    .await;

    // дадим права админу
    let mut admin = RequestUserUpdate::default();
    admin.role = Some(UsersRole::Admin.to_string());

    cl.users_update(admin_id, admin, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // зайдем под owner и создадим команду
    team_id = Uuid::nil();
    cl.login(
        RequestLogin {
            email: req_register_owner.email.clone(),
            password: req_register_owner.password.clone(),
        },
        |result| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_success());
        },
    )
    .await // owner стал частью команды
    .teams_create(rand::request_team(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTeam =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        team_id = resp.team_id;
    })
    .await
    .logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // сейчас owner является членом команды team_id
    // зайдем под other и пригласим кого-то, но у него нет доступа
    cl.login(
        RequestLogin {
            email: req_register_other.email,
            password: req_register_other.password,
        },
        |result| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_success());
        },
    )
    .await
    .teams_invite(team_id, RequestTeamInvite { user_id: owner_id }, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::FORBIDDEN, status_code);
    })
    .await
    .logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // зайдем под owner и пригласим admin
    cl.login(
        RequestLogin {
            email: req_register_owner.email.clone(),
            password: req_register_owner.password.clone(),
        },
        |result| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_success());
        },
    )
    .await
    .teams_invite(team_id, RequestTeamInvite { user_id: admin_id }, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await
    .logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // зайдем под admin и пригласим other, но не owner-а
    cl.login(
        RequestLogin {
            email: req_register_admin.email,
            password: req_register_admin.password,
        },
        |result| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_success());
        },
    )
    .await
    .teams_invite(team_id, RequestTeamInvite { user_id: other_id }, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // выйдем
    .logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // зайдем под owner и пригласим себя же, будет ошибка, т.к. он уже есть среди приглашенных (автоматически)
    cl.login(
        RequestLogin {
            email: req_register_owner.email,
            password: req_register_owner.password,
        },
        |result| {
            let (status_code, _body_str) = result.unwrap();
            assert!(status_code.is_success());
        },
    )
    .await
    .teams_invite(team_id, RequestTeamInvite { user_id: owner_id }, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_server_error());
    })
    .await
    .logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;
}

#[tokio::test]
async fn check_tasks() {
    let ctx = get_context().await;
    let mut cl = Client::new(
        ctx.http_addr.to_string(),
        ctx.ca.to_string(),
        ctx.crt.to_string(),
        ctx.key.to_string(),
        &ctx.db,
    );

    let mut user_id1 = Uuid::nil();
    let mut user_id2 = Uuid::nil();
    let mut team_id = Uuid::nil();
    let mut task_id = Uuid::nil();
    let req_register1 = rand::request_register();
    let req_register2 = rand::request_register();
    let req_team = rand::request_team();
    let mut req_task1 = rand::request_task();
    let mut req_task2 = rand::request_task();
    let req_login1 = RequestLogin {
        email: req_register1.email.clone(),
        password: req_register1.password.clone(),
    };
    let req_login2 = RequestLogin {
        email: req_register2.email.clone(),
        password: req_register2.password.clone(),
    };
    let mut reg_list = RequestTaskData::default();
    reg_list.limit = -1;
    reg_list.offset = -1;

    // проверим на 401
    cl.tasks_list(reg_list.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .tasks_create(rand::request_task(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .tasks_one(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .tasks_update(Uuid::new_v4(), rand::request_task(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .tasks_delete(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .tasks_history(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await;

    // создадим пользователей
    cl.register(req_register1, true, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        user_id1 = resp.uuid;
    })
    .await
    .register(req_register2, true, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        user_id2 = resp.uuid;
    })
    .await;

    // залогинимся, создадим команду из под user_id1, создадим задачу
    cl.login(req_login1.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // ok: user_id1 стал членом команды
    .teams_create(req_team, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_ream_actual: ResponseTeam =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        team_id = resp_ream_actual.team_id;

        req_task1.created_by = user_id1;
        req_task1.team_id = team_id;
        req_task1.assignee_id = None;

        req_task2.created_by = user_id1;
        req_task2.team_id = team_id;
        req_task2.assignee_id = Some(user_id1);
    })
    .await
    .tasks_create(req_task1.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_task: ResponseTask =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        task_id = resp_task.task_id;
    })
    .await // err: с теми же данными
    .tasks_create(req_task1.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, status_code);
    })
    .await;

    // ---------- выйдем под user_id1 и сделаем операции под user_id2, у него доступа не должно быть
    let mut req_task3 = rand::request_task();
    req_task3.created_by = user_id2;
    req_task3.team_id = team_id;
    req_task3.assignee_id = None;

    cl.logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await
    .login(req_login2, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // err - нету прав
    .tasks_create(req_task3.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::FORBIDDEN, status_code);
    })
    .await // err - нету прав
    .tasks_update(task_id, req_task3.clone(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::FORBIDDEN, status_code);
    })
    .await
    .tasks_delete(task_id, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::FORBIDDEN, status_code);
    })
    .await
    .logout(|result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;
    // \ ----------

    // продолжим выполнять под user_id1
    cl.login(req_login1, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());

        reg_list.limit = 100;
        reg_list.offset = 0;
    })
    .await // ok
    .tasks_list(reg_list.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTasksList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(!resp.items.is_empty());
        assert!(resp.total > 0);

        reg_list.limit = 0;
        reg_list.offset = 0;
    })
    .await // ok
    .tasks_list(reg_list.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTasksList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(resp.items.is_empty());
        assert!(resp.total > 0);

        reg_list.limit = -1;
        reg_list.offset = -1;
        reg_list.team_id = Some(Uuid::new_v4());
    })
    .await // ok: применим фильтрацию
    .tasks_list(reg_list.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTasksList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(resp.items.is_empty());
        assert_eq!(0, resp.total);
    })
    .await // err
    .tasks_one(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
    })
    .await // ok
    .tasks_one(task_id, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTask =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(task_id, resp.task_id)
    })
    .await // ok: обновление происходит корректно, т.к. user_id явл. членом команды
    .tasks_update(task_id, req_task2.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTask =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(req_task2.status, resp.status);
        assert_eq!(task_id, resp.task_id);
    })
    .await // err - удалим не известное
    .tasks_delete(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
    })
    .await // ok - член группы может удалить задачу (статус canceled)
    .tasks_delete(task_id, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // ok - считаем историю, должно быть три записи (create, update, delete)
    .tasks_history(task_id, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskHistories =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(3, resp.items.len());
    })
    .await;
}

#[tokio::test]
async fn check_task_comments() {
    let ctx = get_context().await;
    let mut cl = Client::new(
        ctx.http_addr.to_string(),
        ctx.ca.to_string(),
        ctx.crt.to_string(),
        ctx.key.to_string(),
        &ctx.db,
    );

    let mut user_id = Uuid::nil();
    let mut team_id = Uuid::nil();
    let mut task_id = Uuid::nil();
    let req_register = rand::request_register();
    let req_team = rand::request_team();
    let mut req_task = rand::request_task();
    let req_task_comment = rand::request_task_comment();
    let req_login = RequestLogin {
        email: req_register.email.clone(),
        password: req_register.password.clone(),
    };
    let mut task_comment_id1 = Uuid::nil();
    let mut task_comment_id2 = Uuid::nil();

    // проверим на 401
    cl.task_comments_list(Uuid::new_v4(), -1, -1, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .task_comments_create(Uuid::new_v4(), rand::request_task_comment(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .task_comments_delete(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await;

    // создадим пользователя, залогинимся и создадим команду
    cl.register(req_register, true, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        user_id = resp.uuid;
    })
    .await
    .login(req_login, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await
    .teams_create(req_team, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_ream_actual: ResponseTeam =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        team_id = resp_ream_actual.team_id;

        req_task.created_by = user_id;
        req_task.team_id = team_id;
        req_task.assignee_id = None;
    })
    .await
    .tasks_create(req_task, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_task: ResponseTask =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        task_id = resp_task.task_id;
    })
    .await;

    // ok
    cl.task_comments_create(task_id, req_task_comment.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskComment =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);

        assert_ne!(Uuid::nil(), resp.task_comment_id);
        assert_eq!(task_id, resp.task_id);
        assert_eq!(user_id, resp.user_id);
        assert_eq!(req_task_comment.msg, resp.msg);

        task_comment_id1 = resp.task_comment_id;
    })
    .await // ok - с теми же данными
    .task_comments_create(task_id, req_task_comment.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskComment =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);

        assert_ne!(Uuid::nil(), resp.task_comment_id);
        assert_eq!(task_id, resp.task_id);
        assert_eq!(user_id, resp.user_id);
        assert_eq!(req_task_comment.msg, resp.msg);

        task_comment_id2 = resp.task_comment_id;
    })
    .await // ok
    .task_comments_list(task_id, 100, 0, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskCommentsList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(2, resp.items.len());
        assert_eq!(2, resp.total);
    })
    .await // ok
    .task_comments_list(task_id, -1, -1, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskCommentsList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(2, resp.items.len());
        assert_eq!(2, resp.total);
    })
    .await // ok
    .task_comments_list(task_id, 0, 0, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskCommentsList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(resp.items.is_empty());
        assert_eq!(2, resp.total);
    })
    .await // ok: с другим task_id
    .task_comments_list(Uuid::new_v4(), 100, 0, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskCommentsList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(resp.items.is_empty());
        assert_eq!(0, resp.total);
    })
    .await // err
    .task_comments_delete(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_server_error());
    })
    .await // ok
    .task_comments_delete(task_comment_id1, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // ok
    .task_comments_list(task_id, 100, 0, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskCommentsList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(1, resp.items.len());
        assert_eq!(1, resp.total);
    })
    .await
    .task_comments_delete(task_comment_id2, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // ok
    .task_comments_list(task_id, 100, 0, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseTaskCommentsList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(0, resp.items.len());
        assert_eq!(0, resp.total);
    })
    .await;
}

#[tokio::test]
async fn check_users() {
    let ctx = get_context().await;
    let mut cl = Client::new(
        ctx.http_addr.to_string(),
        ctx.ca.to_string(),
        ctx.crt.to_string(),
        ctx.key.to_string(),
        &ctx.db,
    );

    let mut owner_id = Uuid::nil();
    let mut user_id = Uuid::nil();
    let req_register = rand::request_register();
    let mut req_user_create = rand::request_user_create();
    let image_path = rand::create_image("jpg").unwrap();
    let req_login = RequestLogin {
        email: req_register.email.clone(),
        password: req_register.password.clone(),
    };

    req_user_create.name = Some(rand::str());
    req_user_create.role = Some(UsersRole::Admin.to_string());
    req_user_create.avatar = Some(image_path.display().to_string());

    // проверим на 401
    cl.users_list(-1, -1, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .users_create(rand::request_user_create(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .users_update(Uuid::new_v4(), rand::request_user_update(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await
    .users_delete(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, status_code);
    })
    .await;

    // создадим пользователя, залогинимся
    cl.register(req_register, true, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUUID =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        owner_id = resp.uuid;
    })
    .await
    .login(req_login, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // err: пользователя нет
    cl.users_one(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
    })
    .await // err: такого пользователя нет
    .users_delete(Uuid::new_v4(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
    })
    .await // err: такого пользователя нет
    .users_update(Uuid::new_v4(), rand::request_user_update(), |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
    })
    .await // ok
    .users_create(req_user_create.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_user_actual: ResponseUser =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        // cравниваем частями, т.к. типы разные и где-то данных может не быть, а где-то быть
        assert_eq!(req_user_create.email, resp_user_actual.email);
        assert_eq!(req_user_create.name, resp_user_actual.name);
        assert_eq!(req_user_create.role, resp_user_actual.role);
        assert_eq!(
            req_user_create.avatar.is_some(),
            resp_user_actual.avatar.is_some()
        );

        user_id = resp_user_actual.user_id;
    })
    .await // ok
    .users_one(user_id, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await;

    // ok: обновим успешно
    let mut req_user_update = RequestUserUpdate::default();
    req_user_update.name = Some(rand::str());
    req_user_update.role = Some(UsersRole::Null.to_string());
    req_user_update.is_remove_avatar = true;

    cl.users_update(user_id, req_user_update.clone(), |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp_user_actual: ResponseUser =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(req_user_create.email, resp_user_actual.email); // !
        assert_eq!(req_user_update.name, resp_user_actual.name);
        assert!(resp_user_actual.role.is_none());
        assert!(resp_user_actual.avatar.is_none());
    })
    .await // ok: посмотрим что люди есть
    .users_list(0, 0, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let list: ResponseUsersList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert_eq!(list.items.len(), 0);
        assert!(list.total > 0);
    })
    .await // ok: найдем нужное и сравним
    .users_list(-1, -1, |result| {
        let (status_code, body_str) = result.unwrap();
        assert!(status_code.is_success());

        let resp: ResponseUsersList =
            serde_json::from_str(body_str.as_str()).expect(consts::ERR_PARSE_JSON);
        assert!(resp.items.len() > 0);
        assert!(resp.total > 0);
        assert!(
            resp.items
                .iter()
                .find(|item| item.user_id == user_id)
                .is_some()
        );
    })
    .await // ок: удалим успешно
    .users_delete(user_id, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert!(status_code.is_success());
    })
    .await // ok: пользователя не должно быть
    .users_one(user_id, |result| {
        let (status_code, _body_str) = result.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, status_code);
    })
    .await;
}
