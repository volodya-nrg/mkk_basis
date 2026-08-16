mod helpers;

use chrono::Local;
use ctor::ctor;
use helpers::rand;
use mkk_basis::adapter::db::RepositoryError;
use mkk_basis::adapter::db::models::*;
use mkk_basis::adapter::db::postgres::Postgres;
use mkk_basis::adapter::db::postgres::tables::tasks::Status as TaskStatus;
use mkk_basis::adapter::logger;
use sqlx::postgres::PgPoolOptions;
use std::assert_matches;
use std::time::Duration;
use uuid::Uuid;

const DSN: &str =
    "postgres://postgres:postgres@127.0.0.1:5432/postgres?options=-c%20search_path%3Dmkk_basis";

#[ctor(unsafe)]
fn init() {
    logger::init("", "", "", "", true).expect("failed to init logger")
}

// Почему-то лучше подключаться к пулу постоянно.
// Через OnceCell получаю "failed to delete: pool timed out while waiting for an open connection".
async fn get_postgres() -> Postgres {
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::new(3, 0))
        .connect(DSN)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    Postgres::new(pool)
}

#[tokio::test]
async fn check_users() {
    let db = get_postgres().await;
    let time_now = Local::now();

    // err: проверим что запись не находит
    assert_matches!(
        db.tbl_users.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );
    assert_matches!(
        db.tbl_users.get_by_email(rand::email()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: проверим что запись создается
    let mut user_expected: User = rand::user();
    user_expected.user_id = db
        .tbl_users
        .create(user_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что нельзя добавить такую же запись
    assert!(db.tbl_users.create(user_expected.clone()).await.is_err());

    // ok: проверим что запись можно получить и данные их равны
    let user_actual = db
        .tbl_users
        .one(user_expected.user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    user_expected.created_at = user_actual.created_at; // подменим на валидное явно
    user_expected.updated_at = user_actual.updated_at; // подменим на валидное явно
    let user_actual1 = user_actual.clone();
    assert_eq!(user_expected, user_actual);
    assert!(user_expected.created_at.gt(&time_now));

    // ok: проверим что находит по емэйлу
    let user_actual2 = db
        .tbl_users
        .get_by_email(user_actual.email)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert_eq!(user_actual1, user_actual2);

    // ok: проверим что список не пустой
    let (items, total) = db
        .tbl_users
        .list(-1, -1)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let (items, total) = db
        .tbl_users
        .list(0, 0)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим
    assert!(db.tbl_users.update(rand::user()).await.is_err());

    // ok: изменим и проверим пользователя
    let mut user_expected = rand::user();
    user_expected.user_id = user_actual.user_id; // подменим на валидное явно
    db.tbl_users
        .update(user_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let user_actual = db
        .tbl_users
        .one(user_expected.user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    user_expected.created_at = user_actual.created_at; // подменим на валидное явно
    user_expected.updated_at = user_actual.updated_at; // подменим на валидное явно
    assert_eq!(user_expected, user_actual);
    assert!(user_actual.updated_at.gt(&user_actual.created_at));

    // err: удалим не известного пользователя
    assert!(db.tbl_users.delete(Uuid::new_v4()).await.is_err());

    // ok: удалим пользователя
    db.tbl_users
        .delete(user_actual.user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // ok: не нашли пользователя, как и задумано
    assert_matches!(
        db.tbl_users.one(user_actual.user_id).await,
        Err(RepositoryError::NotFoundRow)
    );
}

#[tokio::test]
async fn check_teams() {
    let db = get_postgres().await;
    let time_now = Local::now();

    // ok: создадим пользователя
    let user_id = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что команду не находит
    assert_matches!(
        db.tbl_teams.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // err: попытаемся создать команду, но такой нет
    assert!(db.tbl_teams.create(rand::team()).await.is_err());

    // ok: проверим что создается
    let mut team_expected: Team = rand::team();
    team_expected.created_by = user_id; // зависит от user-а
    team_expected.team_id = db
        .tbl_teams
        .create(team_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что нельзя добавить такую же запись
    assert!(db.tbl_teams.create(team_expected.clone()).await.is_err());

    // ok: проверим что можно получить и их данные равны
    let team_actual = db
        .tbl_teams
        .one(team_expected.team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    team_expected.created_at = team_actual.created_at; // подменим на валидное явно
    team_expected.updated_at = team_actual.updated_at; // подменим на валидное явно
    assert_eq!(team_expected, team_actual);
    assert!(team_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let (items, total) = db
        .tbl_teams
        .list(-1, -1)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let (items, total) = db
        .tbl_teams
        .list(0, 0)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим неизвестного
    assert!(db.tbl_teams.update(rand::team()).await.is_err());

    // ok: изменим и проверим
    let mut team_expected = rand::team();
    team_expected.team_id = team_actual.team_id; // подменим на валидное явно
    team_expected.created_by = user_id;
    db.tbl_teams
        .update(team_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let team_actual = db
        .tbl_teams
        .one(team_expected.team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    team_expected.created_at = team_actual.created_at; // подменим на валидное явно
    team_expected.updated_at = team_actual.updated_at; // подменим на валидное явно
    assert_eq!(team_expected, team_actual);
    assert!(team_actual.updated_at.gt(&team_actual.created_at));

    // err: попытаемся удалить
    assert!(db.tbl_teams.delete(Uuid::new_v4()).await.is_err());

    // err: нельзя удалить пользователя пока есть привязанная команда
    assert!(db.tbl_users.delete(user_id).await.is_err());

    // ok: удалим
    db.tbl_teams
        .delete(team_actual.team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_teams.one(team_actual.team_id).await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_users
        .delete(user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
}

#[tokio::test]
async fn check_team_members() {
    let db = get_postgres().await;
    let time_now = Local::now();

    // ok: создадим команду и пользователя
    let user_id = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut team = rand::team();
    team.created_by = user_id;
    let team_id = db
        .tbl_teams
        .create(team)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что не находит
    assert_matches!(
        db.tbl_team_members
            .one(Uuid::new_v4(), Uuid::new_v4())
            .await,
        Err(RepositoryError::NotFoundRow)
    );

    // err: попытаемся создать, но связанных данных нет
    assert!(
        db.tbl_team_members
            .create(rand::team_member())
            .await
            .is_err()
    );

    // ok: проверим что создается
    let mut team_member_expected = rand::team_member();
    team_member_expected.team_id = team_id; // зависит от team-а
    team_member_expected.user_id = user_id; // зависит от user-а
    db.tbl_team_members
        .create(team_member_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что нельзя добавить такую же запись
    assert!(
        db.tbl_team_members
            .create(team_member_expected.clone())
            .await
            .is_err()
    );

    // ok: проверим что можно получить и их данные равны
    let team_member_actual = db
        .tbl_team_members
        .one(team_member_expected.team_id, team_member_expected.user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    team_member_expected.created_at = team_member_actual.created_at; // подменим на валидное явно
    assert_eq!(team_member_expected, team_member_actual);
    assert!(team_member_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let result = db
        .tbl_team_members
        .all()
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(!result.is_empty());

    // err: попытаемся удалить
    assert!(
        db.tbl_team_members
            .delete(Uuid::new_v4(), Uuid::new_v4())
            .await
            .is_err()
    );

    // ok: удалим
    db.tbl_team_members
        .delete(team_member_actual.team_id, team_member_actual.user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_team_members
            .one(team_member_actual.team_id, team_member_actual.user_id)
            .await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_teams
        .delete(team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // проверим каскадное удаление, относительно team_id
    let user_id = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut team = rand::team();
    team.created_by = user_id;

    let team_id: Uuid = db
        .tbl_teams
        .create(team)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut team_member_expected = rand::team_member();
    team_member_expected.team_id = team_id;
    team_member_expected.user_id = user_id;

    db.tbl_team_members
        .create(team_member_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_teams
        .delete(team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert_matches!(
        db.tbl_team_members
            .one(team_member_expected.team_id, team_member_expected.user_id)
            .await,
        Err(RepositoryError::NotFoundRow)
    );
    db.tbl_users
        .delete(user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
}

#[tokio::test]
async fn check_tasks() {
    let db = get_postgres().await;
    let time_now = Local::now();

    // ok: создадим пользователя и команду
    let user_id: Uuid = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut team = rand::team();
    team.created_by = user_id;
    let team_id: Uuid = db
        .tbl_teams
        .create(team)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что задачу не находит
    assert_matches!(
        db.tbl_tasks.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // err: попытаемся создать задачу, но такой нет
    assert!(db.tbl_tasks.create(rand::task()).await.is_err());

    // ok: проверим что создается
    let mut task_expected = rand::task();
    task_expected.team_id = team_id; // зависит от team
    task_expected.created_by = user_id; // зависит от user-а
    task_expected.assignee_id = Some(user_id); // зависит от user-а
    task_expected.status = TaskStatus::Start.to_string();
    task_expected.task_id = db
        .tbl_tasks
        .create(task_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что нельзя добавить такую же запись
    assert!(db.tbl_tasks.create(task_expected.clone()).await.is_err());

    // ok: проверим что можно получить и их данные равны
    let task_actual = db
        .tbl_tasks
        .one(task_expected.task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    task_expected.created_at = task_actual.created_at; // подменим на валидное явно
    task_expected.updated_at = task_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_expected, task_actual);
    assert!(task_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let (items, total) = db
        .tbl_tasks
        .list(-1, -1)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let (items, total) = db
        .tbl_tasks
        .list(0, 0)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим неизвестного
    assert!(db.tbl_tasks.update(rand::task()).await.is_err());

    // ok: изменим и проверим
    let mut task_expected = rand::task();
    task_expected.task_id = task_actual.task_id;
    task_expected.team_id = team_id; // зависит от team
    task_expected.created_by = user_id; // зависит от user-а
    task_expected.assignee_id = None; // зависит от user-а
    task_expected.status = TaskStatus::Cancelled.to_string();
    db.tbl_tasks
        .update(task_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    let task_actual = db
        .tbl_tasks
        .one(task_expected.task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    task_expected.created_at = task_actual.created_at; // подменим на валидное явно
    task_expected.updated_at = task_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_expected, task_actual);
    assert!(task_actual.updated_at.gt(&task_actual.created_at));

    // err: попытаемся удалить
    assert!(db.tbl_tasks.delete(Uuid::new_v4()).await.is_err());

    // err: нельзя удалить пока есть привязанная сущность
    assert!(db.tbl_users.delete(user_id).await.is_err());

    // err: нельзя удалить пока есть привязанная сущность
    assert!(db.tbl_teams.delete(team_id).await.is_err());

    // ok: удалим
    db.tbl_tasks
        .delete(task_actual.task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_tasks.one(task_actual.task_id).await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_teams
        .delete(task_actual.team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
}

#[tokio::test]
async fn check_task_histories() {
    let db = get_postgres().await;
    let time_now = Local::now();

    // ok: создадим зависимости
    let user_id = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let user_id2 = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut team: Team = rand::team();
    team.created_by = user_id;
    let team_id = db
        .tbl_teams
        .create(team)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut task: Task = rand::task();
    task.team_id = team_id;
    task.created_by = user_id;
    task.assignee_id = None;
    task.status = TaskStatus::Todo.to_string();
    let task_id = db
        .tbl_tasks
        .create(task)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что не находит
    assert_matches!(
        db.tbl_task_histories.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // err: попытаемся создать
    assert!(
        db.tbl_task_histories
            .create(rand::task_history())
            .await
            .is_err()
    );

    // ok: проверим что создается
    let mut task_history_expected = rand::task_history();
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id;
    task_history_expected.task_history_id = db
        .tbl_task_histories
        .create(task_history_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что нельзя добавить такую же запись
    assert!(
        db.tbl_task_histories
            .create(task_history_expected.clone())
            .await
            .is_err()
    );

    // ok: проверим что можно получить и их данные равны
    let task_history_actual = db
        .tbl_task_histories
        .one(task_history_expected.task_history_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    task_history_expected.created_at = task_history_actual.created_at; // подменим на валидное явно
    assert_eq!(task_history_expected, task_history_actual);
    assert!(task_history_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let (items, total) = db
        .tbl_task_histories
        .list(-1, -1)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let (items, total) = db
        .tbl_task_histories
        .list(0, 0)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // получим список относительно task_id
    let items = db
        .tbl_task_histories
        .get_by_task_id(task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert_eq!(1, items.len());

    // err: изменим неизвестного
    assert!(
        db.tbl_task_histories
            .update(rand::task_history())
            .await
            .is_err()
    );

    // ok: изменим и проверим
    let mut task_history_expected = rand::task_history();
    task_history_expected.task_history_id = task_history_actual.task_history_id;
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id2;

    db.tbl_task_histories
        .update(task_history_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let task_history_actual = db
        .tbl_task_histories
        .one(task_history_expected.task_history_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    task_history_expected.created_at = task_history_actual.created_at; // подменим на валидное явно
    assert_eq!(task_history_expected, task_history_actual);

    // err: попытаемся удалить
    assert!(db.tbl_task_histories.delete(Uuid::new_v4()).await.is_err());

    // ok: удалим
    db.tbl_task_histories
        .delete(task_history_actual.task_history_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_task_histories
            .one(task_history_actual.task_history_id)
            .await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_tasks
        .delete(task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_teams
        .delete(team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id2)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // проверим каскадное удаление относительно task_id
    let user_id = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut team: Team = rand::team();
    team.created_by = user_id;
    let team_id = db
        .tbl_teams
        .create(team)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut task: Task = rand::task();
    task.team_id = team_id;
    task.created_by = user_id;
    task.assignee_id = None;
    task.status = TaskStatus::Todo.to_string();
    let task_id = db
        .tbl_tasks
        .create(task.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    let mut task_history_expected = rand::task_history();
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id;
    let task_history_id = db
        .tbl_task_histories
        .create(task_history_expected)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_tasks
        .delete(task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert_matches!(
        db.tbl_task_histories.one(task_history_id).await,
        Err(RepositoryError::NotFoundRow)
    );

    // проверим каскадное удаление относительно user_id2
    let task_id = db
        .tbl_tasks
        .create(task)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let user_id2: Uuid = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut task_history_expected = rand::task_history();
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id2;
    let task_history_id = db
        .tbl_task_histories
        .create(task_history_expected)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id2)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert_matches!(
        db.tbl_task_histories.one(task_history_id).await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_tasks
        .delete(task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_teams
        .delete(team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
}

#[tokio::test]
async fn check_task_comments() {
    let db = get_postgres().await;
    let time_now = Local::now();

    // ok: создадим зависимости
    let user_id: Uuid = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let user_id2: Uuid = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut team = rand::team();
    team.created_by = user_id;
    let team_id: Uuid = db
        .tbl_teams
        .create(team)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut task = rand::task();
    task.team_id = team_id;
    task.created_by = user_id;
    task.assignee_id = None;
    task.status = TaskStatus::Todo.to_string();
    let task_id = db
        .tbl_tasks
        .create(task)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // err: проверим что не находит
    assert_matches!(
        db.tbl_task_comments.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // err: попытаемся создать, отсутствуют зависимости
    assert!(
        db.tbl_task_comments
            .create(rand::task_comment())
            .await
            .is_err()
    );

    // ok: проверим что создается
    let mut task_comment_expected = rand::task_comment();
    task_comment_expected.task_id = task_id;
    task_comment_expected.user_id = user_id;
    task_comment_expected.task_comment_id = db
        .tbl_task_comments
        .create(task_comment_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // ok: проверим что можно добавить такую же запись
    db.tbl_task_comments
        .create(task_comment_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // ok: проверим что можно получить и их данные равны
    let task_comment_actual = db
        .tbl_task_comments
        .one(task_comment_expected.task_comment_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    task_comment_expected.created_at = task_comment_actual.created_at; // подменим на валидное явно
    task_comment_expected.updated_at = task_comment_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_comment_expected, task_comment_actual);
    assert!(task_comment_expected.created_at.gt(&time_now));
    assert!(
        task_comment_expected
            .created_at
            .eq(&task_comment_expected.updated_at)
    );

    // ok: проверим что список не пустой
    let (items, total) = db
        .tbl_task_comments
        .list(-1, -1)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let (items, total) = db
        .tbl_task_comments
        .list(0, 0)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим неизвестного
    assert!(
        db.tbl_task_comments
            .update(rand::task_comment())
            .await
            .is_err()
    );

    // ok: изменим и проверим
    let mut task_comment_expected = rand::task_comment();
    task_comment_expected.task_comment_id = task_comment_actual.task_comment_id;
    task_comment_expected.task_id = task_id;
    task_comment_expected.user_id = user_id2;

    db.tbl_task_comments
        .update(task_comment_expected.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let task_comment_actual = db
        .tbl_task_comments
        .one(task_comment_expected.task_comment_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    task_comment_expected.created_at = task_comment_actual.created_at; // подменим на валидное явно
    task_comment_expected.updated_at = task_comment_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_comment_expected, task_comment_actual);
    assert!(
        task_comment_expected
            .updated_at
            .gt(&task_comment_expected.created_at)
    );

    // err: попытаемся удалить
    assert!(db.tbl_task_comments.delete(Uuid::new_v4()).await.is_err());

    // ok: удалим
    db.tbl_task_comments
        .delete(task_comment_actual.task_comment_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_task_comments
            .one(task_comment_actual.task_comment_id)
            .await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_tasks
        .delete(task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_teams
        .delete(team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id2)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    // проверим каскадное удаление относительно task_id
    let user_id: Uuid = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut team: Team = rand::team();
    team.created_by = user_id;
    let team_id: Uuid = db
        .tbl_teams
        .create(team)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut task: Task = rand::task();
    task.team_id = team_id;
    task.created_by = user_id;
    task.assignee_id = None;
    task.status = TaskStatus::Done.to_string();
    let task_id = db
        .tbl_tasks
        .create(task.clone())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut task_comment_expected = rand::task_comment();
    task_comment_expected.task_id = task_id;
    task_comment_expected.user_id = user_id;
    let task_comment_id = db
        .tbl_task_comments
        .create(task_comment_expected)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_tasks
        .delete(task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert_matches!(
        db.tbl_task_comments.one(task_comment_id).await,
        Err(RepositoryError::NotFoundRow)
    );

    // проверим каскадное удаление относительно user_id2
    let task_id = db
        .tbl_tasks
        .create(task)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let user_id2: Uuid = db
        .tbl_users
        .create(rand::user())
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    let mut task_comment_expected = rand::task_comment();
    task_comment_expected.task_id = task_id;
    task_comment_expected.user_id = user_id2;
    let task_comment_id = db
        .tbl_task_comments
        .create(task_comment_expected)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id2)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    assert_matches!(
        db.tbl_task_comments.one(task_comment_id).await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_tasks
        .delete(task_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_teams
        .delete(team_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
    db.tbl_users
        .delete(user_id)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));
}
