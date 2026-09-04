mod helpers;

use chrono::{DateTime, Local};
use ctor::ctor;
use sqlx::postgres::PgPoolOptions;
use sqlx::testing::TestTermination;
use std::assert_matches;
use std::string::ToString;
use std::time::Duration;
use uuid::Uuid;

use mkk_basis::adapter::{
    db::{
        errors::RepositoryError,
        models::TaskData,
        postgres::{
            Postgres as PostgresService, tables::tasks::Status as TaskStatus,
            tables::users::Role as UserRoles,
        },
    },
    logger,
};

use helpers::rand;

const DSN: &str =
    "postgres://postgres:postgres@127.0.0.1:5432/postgres?options=-c%20search_path%3Dmkk_basis";

#[ctor(unsafe)]
fn init() {
    logger::init(String::new(), String::new(), String::new(), None, true).unwrap()
}

// Почему-то лучше подключаться к пулу постоянно.
// Через OnceCell получаю "failed to delete: pool timed out while waiting for an open connection".
async fn get_postgres() -> (PostgresService, DateTime<Local>) {
    let time_now = Local::now();
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::new(3, 0))
        .connect(DSN)
        .await
        .unwrap();
    (PostgresService::new(pool), time_now)
}

#[tokio::test]
async fn check_users() {
    let (db, time_now) = get_postgres().await;

    // err: проверим что запись не находит
    assert_matches!(
        db.tbl_users.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );
    assert_matches!(
        db.tbl_users.by_email(rand::email()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: проверим что запись создается
    let mut user_expected = rand::user();
    user_expected.user_id = db.tbl_users.create(user_expected.clone()).await.unwrap();
    assert!(!user_expected.user_id.is_nil());

    // err: проверим что нельзя добавить такую же запись
    assert!(db.tbl_users.create(user_expected.clone()).await.is_err());

    // ok: проверим что запись можно получить и данные их равны
    let mut user_actual = db.tbl_users.one(user_expected.user_id).await.unwrap();
    assert!(user_actual.created_at.gt(&time_now));
    assert_eq!(user_actual.created_at, user_actual.updated_at);
    user_expected.created_at = user_actual.created_at; // подменим на валидное явно
    user_expected.updated_at = user_actual.updated_at; // подменим на валидное явно
    assert_eq!(user_expected, user_actual);

    // ok: проверим что находит по е-мэйлу
    assert_eq!(
        user_actual.clone(),
        db.tbl_users.by_email(user_actual.email).await.unwrap()
    );

    // ok: проверим что список не пустой
    let (items, total) = db.tbl_users.list(-1, -1).await.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат, но общее кол-во есть
    let (items, total) = db.tbl_users.list(0, 0).await.unwrap();
    assert!(items.is_empty());
    assert!(total > 0);

    // err: изменим не понятно у кого
    assert!(db.tbl_users.update(rand::user()).await.is_err());

    // ok: изменим и проверим пользователя
    user_expected = rand::user();
    user_expected.user_id = user_actual.user_id; // подменим на валидное явно
    user_expected.role = Some(UserRoles::Admin.to_string());
    assert!(
        db.tbl_users
            .update(user_expected.clone())
            .await
            .is_success()
    );
    user_actual = db.tbl_users.one(user_expected.user_id).await.unwrap();
    assert!(user_actual.updated_at.gt(&user_actual.created_at));
    user_expected.created_at = user_actual.created_at; // подменим на валидное явно
    user_expected.updated_at = user_actual.updated_at; // подменим на валидное явно
    assert_eq!(user_expected, user_actual);

    // проверим роли
    user_expected.role = Some("".to_string());
    assert!(db.tbl_users.update(user_expected.clone()).await.is_err());
    user_expected.role = Some(UserRoles::Null.to_string());
    assert!(
        db.tbl_users
            .update(user_expected.clone())
            .await
            .is_success()
    );
    assert!(
        db.tbl_users
            .one(user_expected.user_id)
            .await
            .unwrap()
            .role
            .is_none()
    );
    user_expected.role = Some(UserRoles::Moder.to_string());
    assert!(
        db.tbl_users
            .update(user_expected.clone())
            .await
            .is_success()
    );
    user_expected.role = None;
    assert!(
        db.tbl_users
            .update(user_expected.clone())
            .await
            .is_success()
    );
    assert!(
        db.tbl_users
            .one(user_expected.user_id)
            .await
            .unwrap()
            .role
            .is_none()
    );

    // err: удалим не известного пользователя
    assert!(db.tbl_users.delete(Uuid::new_v4()).await.is_err());

    // ok: удалим пользователя
    assert!(db.tbl_users.delete(user_actual.user_id).await.is_success());

    // ok: не нашли пользователя, как и задумано
    assert_matches!(
        db.tbl_users.one(user_actual.user_id).await,
        Err(RepositoryError::NotFoundRow)
    );
}

#[tokio::test]
async fn check_teams() {
    let (db, time_now) = get_postgres().await;
    let user_id = db.tbl_users.create(rand::user()).await.unwrap();

    // err: проверим что команду не находит
    assert_matches!(
        db.tbl_teams.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // err: попытаемся создать команду, но такого пользователя нет
    assert!(db.tbl_teams.create(rand::team()).await.is_err());

    // ok: проверим что создается, с существующим пользователем
    let mut team_expected = rand::team();
    team_expected.created_by = user_id;
    team_expected.team_id = db.tbl_teams.create(team_expected.clone()).await.unwrap();
    assert!(!team_expected.team_id.is_nil());

    // err: проверим что нельзя добавить такую же запись, т.к. поле "name" уникально
    assert!(db.tbl_teams.create(team_expected.clone()).await.is_err());

    // ok: проверим что можно получить и данные равны
    let mut team_actual = db.tbl_teams.one(team_expected.team_id).await.unwrap();
    assert!(team_actual.created_at.gt(&time_now));
    assert_eq!(team_actual.created_at, team_actual.updated_at);
    team_expected.created_at = team_actual.created_at; // подменим на валидное явно
    team_expected.updated_at = team_actual.updated_at; // подменим на валидное явно
    assert_eq!(team_expected, team_actual);

    // ok: проверим что список не пустой
    let (items, total) = db.tbl_teams.list(-1, -1).await.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат, но общее кол-во есть
    let (items, total) = db.tbl_teams.list(0, 0).await.unwrap();
    assert!(items.is_empty());
    assert!(total > 0);

    // err: изменим неизвестного
    assert!(db.tbl_teams.update(rand::team()).await.is_err());

    // ok: изменим и проверим
    team_expected = rand::team();
    team_expected.team_id = team_actual.team_id; // подменим на валидное явно
    team_expected.created_by = user_id;
    assert!(
        db.tbl_teams
            .update(team_expected.clone())
            .await
            .is_success()
    );
    team_actual = db.tbl_teams.one(team_expected.team_id).await.unwrap();
    assert!(team_actual.updated_at.gt(&team_actual.created_at));
    team_expected.created_at = team_actual.created_at; // подменим на валидное явно
    team_expected.updated_at = team_actual.updated_at; // подменим на валидное явно
    assert_eq!(team_expected, team_actual);

    // err: попытаемся удалить не известную команду
    assert!(db.tbl_teams.delete(Uuid::new_v4()).await.is_err());

    // err: нельзя удалить пользователя пока есть привязанная команда
    assert!(db.tbl_users.delete(user_id).await.is_err());

    // ok: проверим что появилась запись в таблице team_members
    let team_member = db
        .tbl_team_members
        .one(team_actual.team_id, team_actual.created_by)
        .await
        .unwrap();
    assert_eq!(team_actual.team_id, team_member.team_id);
    assert_eq!(team_actual.created_by, team_member.user_id);
    assert!(team_member.created_at.gt(&time_now));

    // ok: удалим
    assert!(db.tbl_teams.delete(team_actual.team_id).await.is_success());

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_teams.one(team_actual.team_id).await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: каскадно удалилась
    assert_matches!(
        db.tbl_team_members
            .one(team_actual.team_id, team_actual.created_by)
            .await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_users.delete(user_id).await.unwrap();
}

#[tokio::test]
async fn check_team_members() {
    let (db, time_now) = get_postgres().await;

    // ok: создадим пользователя и команду
    let user_id1 = db.tbl_users.create(rand::user()).await.unwrap();
    let user_id2 = db.tbl_users.create(rand::user()).await.unwrap();
    let mut team = rand::team();
    team.created_by = user_id1;
    let team_id = db.tbl_teams.create(team).await.unwrap();

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

    // ok: проверим что НЕ создается, т.к. запись уже должна быть при создании записи о team
    let mut team_member_expected = rand::team_member();
    team_member_expected.team_id = team_id;
    team_member_expected.user_id = user_id1;
    assert!(
        db.tbl_team_members
            .create(team_member_expected.clone())
            .await
            .is_err()
    );

    // ok: явно добавим запись для user2
    team_member_expected.user_id = user_id2;
    assert!(
        db.tbl_team_members
            .create(team_member_expected.clone())
            .await
            .is_success()
    );

    // ok: проверим что можно получить и их данные равны
    let team_member_actual = db
        .tbl_team_members
        .one(team_member_expected.team_id, team_member_expected.user_id)
        .await
        .unwrap();
    assert!(team_member_actual.created_at.gt(&time_now));
    team_member_expected.created_at = team_member_actual.created_at; // подменим на валидное явно
    assert_eq!(team_member_expected, team_member_actual);

    // ok: проверим что список не пустой
    assert!(!db.tbl_team_members.all().await.unwrap().is_empty());

    // err: попытаемся удалить не известное
    assert!(
        db.tbl_team_members
            .delete(Uuid::new_v4(), Uuid::new_v4())
            .await
            .is_err()
    );

    // ok: удалим пользователя, запись о члене должно удалится каскадно
    db.tbl_users.delete(user_id2).await.unwrap();

    // ok: записи не должно быть
    assert_matches!(
        db.tbl_team_members
            .one(team_member_actual.team_id, team_member_actual.user_id)
            .await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: первая запись должна еще быть
    assert!(
        db.tbl_team_members
            .one(team_id, user_id1)
            .await
            .is_success()
    );

    // ok: удалим team, запись о member тоже должна исчезнуть
    db.tbl_teams.delete(team_id).await.unwrap();

    // ok: записи не должно быть
    assert_matches!(
        db.tbl_team_members.one(team_id, user_id1).await,
        Err(RepositoryError::NotFoundRow)
    );

    // почистим
    db.tbl_users.delete(user_id1).await.unwrap();
}

#[tokio::test]
async fn check_tasks() {
    let (db, time_now) = get_postgres().await;

    // ok: создадим пользователя и команду
    let user_id1 = db.tbl_users.create(rand::user()).await.unwrap();
    let user_id2 = db.tbl_users.create(rand::user()).await.unwrap();
    let mut team = rand::team();
    team.created_by = user_id1;
    let team_id: Uuid = db.tbl_teams.create(team).await.unwrap();

    // err: проверим что задачу не находит
    assert_matches!(
        db.tbl_tasks.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // err: попытаемся создать задачу, но связанных данных нет
    assert!(db.tbl_tasks.create(rand::task()).await.is_err());

    // ok: проверим что создается
    let mut task_expected = rand::task();
    task_expected.team_id = team_id; // зависит от team
    task_expected.created_by = user_id1; // зависит от user-а
    task_expected.assignee_id = Some(user_id2); // зависит от user-а
    task_expected.status = TaskStatus::Start.to_string(); // все таки надо явно
    task_expected.task_id = db.tbl_tasks.create(task_expected.clone()).await.unwrap();
    assert!(!task_expected.task_id.is_nil());

    // err: проверим что нельзя добавить такую же запись, т.к. created_by, team_id уникальны
    assert!(db.tbl_tasks.create(task_expected.clone()).await.is_err());

    // ok: проверим что можно получить и их данные равны
    let mut task_actual = db.tbl_tasks.one(task_expected.task_id).await.unwrap();
    assert!(task_actual.created_at.gt(&time_now));
    assert_eq!(task_actual.created_at, task_actual.updated_at);
    task_expected.created_at = task_actual.created_at; // подменим на валидное явно
    task_expected.updated_at = task_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_expected, task_actual);

    let mut task_data = TaskData::default();
    task_data.limit = -1;
    task_data.offset = -1;

    // ok: проверим что список не пустой
    let (mut items, mut total) = db.tbl_tasks.list(task_data.clone()).await.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    task_data.limit = 0;
    task_data.offset = 0;
    (items, total) = db.tbl_tasks.list(task_data.clone()).await.unwrap();
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // проверим фильтрацию списка
    // - список пустой, т.к. не известный team_id
    task_data.limit = -1;
    task_data.offset = -1;
    task_data.team_id = Some(Uuid::new_v4());
    (items, total) = db.tbl_tasks.list(task_data.clone()).await.unwrap();
    assert!(items.is_empty());
    assert_eq!(0, total);

    // - список не пустой, т.к. известный team_id
    task_data.team_id = Some(team_id);
    (items, total) = db.tbl_tasks.list(task_data.clone()).await.unwrap();
    assert!(!items.is_empty());
    assert_eq!(1, total);

    // - список пустой, т.к. известный team_id и не известный assignee_id
    task_data.assignee_id = Some(Uuid::new_v4());
    (items, total) = db.tbl_tasks.list(task_data.clone()).await.unwrap();
    assert!(items.is_empty());
    assert_eq!(0, total);

    // - список не пустой, т.к. известный team_id и известный assignee_id
    task_data.assignee_id = Some(user_id2);
    (items, total) = db.tbl_tasks.list(task_data.clone()).await.unwrap();
    assert!(!items.is_empty());
    assert_eq!(1, total);

    // - список пустой, т.к. известный team_id, assignee_id и не тот статус
    task_data.status = Some(TaskStatus::Cancelled.to_string());
    (items, total) = db.tbl_tasks.list(task_data.clone()).await.unwrap();
    assert!(items.is_empty());
    assert_eq!(0, total);

    // - список не пустой, т.к. известный team_id, assignee_id и верный статус
    task_data.status = Some(TaskStatus::Start.to_string());
    (items, total) = db.tbl_tasks.list(task_data.clone()).await.unwrap();
    assert!(!items.is_empty());
    assert_eq!(1, total);

    // - список пустой, т.к. известный team_id, assignee_id и верный статус, но лимит 0
    task_data.limit = 0;
    task_data.status = Some(TaskStatus::Start.to_string());
    (items, total) = db.tbl_tasks.list(task_data).await.unwrap();
    assert!(items.is_empty());
    assert_eq!(1, total);

    // err: изменим неизвестного
    assert!(db.tbl_tasks.update(rand::task()).await.is_err());

    // удалим user2 и посмотрим что assignee_id=None;
    db.tbl_users.delete(user_id2).await.unwrap();
    assert!(
        db.tbl_tasks
            .one(task_actual.task_id)
            .await
            .unwrap()
            .assignee_id
            .is_none()
    );

    // ok: изменим и проверим
    task_expected = rand::task();
    task_expected.task_id = task_actual.task_id;
    task_expected.team_id = team_id; // зависит от team
    task_expected.created_by = user_id1; // зависит от user-а
    task_expected.assignee_id = None; // зависит от user-а
    task_expected.status = TaskStatus::Cancelled.to_string();
    assert!(
        db.tbl_tasks
            .update(task_expected.clone())
            .await
            .is_success()
    );

    task_actual = db.tbl_tasks.one(task_expected.task_id).await.unwrap();
    assert!(task_actual.updated_at.gt(&task_actual.created_at));
    task_expected.created_at = task_actual.created_at; // подменим на валидное явно
    task_expected.updated_at = task_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_expected, task_actual);

    // err: попытаемся удалить не известное
    assert!(db.tbl_tasks.delete(Uuid::new_v4()).await.is_err());

    // err: нельзя удалить пока есть привязанная сущность
    assert!(db.tbl_users.delete(user_id1).await.is_err());

    // err: нельзя удалить пока есть привязанная сущность
    assert!(db.tbl_teams.delete(team_id).await.is_err());

    // ok: удалим
    assert!(db.tbl_tasks.delete(task_actual.task_id).await.is_success());

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_tasks.one(task_actual.task_id).await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_teams.delete(task_actual.team_id).await.unwrap();
    db.tbl_users.delete(user_id1).await.unwrap();
}

#[tokio::test]
async fn check_task_histories() {
    let (db, time_now) = get_postgres().await;

    // ok: создадим зависимости
    let user_id1 = db.tbl_users.create(rand::user()).await.unwrap();
    let user_id2 = db.tbl_users.create(rand::user()).await.unwrap();
    let mut team = rand::team();
    team.created_by = user_id1;
    let team_id = db.tbl_teams.create(team).await.unwrap();
    let mut task = rand::task();
    task.team_id = team_id;
    task.created_by = user_id1;
    task.assignee_id = None;
    task.status = TaskStatus::Todo.to_string();
    let task_id = db.tbl_tasks.create(task).await.unwrap();

    // err: проверим что не находит
    assert_matches!(
        db.tbl_task_histories.one(Uuid::new_v4()).await,
        Err(RepositoryError::NotFoundRow)
    );

    // err: попытаемся создать, но связей нет
    assert!(
        db.tbl_task_histories
            .create(rand::task_history())
            .await
            .is_err()
    );

    // ok: проверим что создается
    let mut task_history_expected = rand::task_history();
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id1;
    task_history_expected.task_history_id = db
        .tbl_task_histories
        .create(task_history_expected.clone())
        .await
        .unwrap();
    assert!(!task_history_expected.task_history_id.is_nil());

    // ok: проверим что можно получить и их данные равны
    let task_history_actual = db
        .tbl_task_histories
        .one(task_history_expected.task_history_id)
        .await
        .unwrap();
    assert!(task_history_actual.created_at.gt(&time_now));
    task_history_expected.created_at = task_history_actual.created_at; // подменим на валидное явно
    assert_eq!(task_history_expected, task_history_actual);

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
    let items = db.tbl_task_histories.by_task_id(task_id).await.unwrap();
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

    assert!(
        db.tbl_task_histories
            .update(task_history_expected.clone())
            .await
            .is_success()
    );

    let task_history_actual = db
        .tbl_task_histories
        .one(task_history_expected.task_history_id)
        .await
        .unwrap();
    task_history_expected.created_at = task_history_actual.created_at; // подменим на валидное явно
    assert_eq!(task_history_expected, task_history_actual);

    // err: попытаемся удалить не известное
    assert!(db.tbl_task_histories.delete(Uuid::new_v4()).await.is_err());
    // err: попытаемся удалить задачу
    assert!(db.tbl_tasks.delete(task_id).await.is_err());
    // err: попытаемся удалить пользователя
    assert!(db.tbl_users.delete(user_id2).await.is_err());

    // ok: удалим
    assert!(
        db.tbl_task_histories
            .delete(task_history_actual.task_history_id)
            .await
            .is_success()
    );

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_task_histories
            .one(task_history_actual.task_history_id)
            .await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_tasks.delete(task_id).await.unwrap();
    db.tbl_teams.delete(team_id).await.unwrap();
    db.tbl_users.delete(user_id1).await.unwrap();
    db.tbl_users.delete(user_id2).await.unwrap();
}

#[tokio::test]
async fn check_task_comments() {
    let (db, time_now) = get_postgres().await;

    // ok: создадим зависимости
    let mut user_id1 = db.tbl_users.create(rand::user()).await.unwrap();
    let mut user_id2 = db.tbl_users.create(rand::user()).await.unwrap();
    let mut team = rand::team();
    team.created_by = user_id1;
    let mut team_id = db.tbl_teams.create(team).await.unwrap();
    let mut task = rand::task();
    task.team_id = team_id;
    task.created_by = user_id1;
    task.assignee_id = None;
    task.status = TaskStatus::Todo.to_string();
    let mut task_id1 = db.tbl_tasks.create(task).await.unwrap();

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
    task_comment_expected.task_id = task_id1;
    task_comment_expected.user_id = user_id1;
    task_comment_expected.task_comment_id = db
        .tbl_task_comments
        .create(task_comment_expected.clone())
        .await
        .unwrap();
    assert!(!task_comment_expected.task_comment_id.is_nil());

    // ok: проверим что можно получить и их данные равны
    let task_comment_actual = db
        .tbl_task_comments
        .one(task_comment_expected.task_comment_id)
        .await
        .unwrap();
    assert!(task_comment_actual.created_at.gt(&time_now));
    assert_eq!(
        task_comment_actual.created_at,
        task_comment_actual.updated_at
    );
    task_comment_expected.created_at = task_comment_actual.created_at; // подменим на валидное явно
    task_comment_expected.updated_at = task_comment_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_comment_expected, task_comment_actual);

    // ok: проверим что список не пустой
    let (items, total) = db.tbl_task_comments.list(task_id1, -1, -1).await.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let (items, total) = db.tbl_task_comments.list(task_id1, 0, 0).await.unwrap();
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
    task_comment_expected.task_id = task_id1;
    task_comment_expected.user_id = user_id2;

    assert!(
        db.tbl_task_comments
            .update(task_comment_expected.clone())
            .await
            .is_success()
    );
    let task_comment_actual = db
        .tbl_task_comments
        .one(task_comment_expected.task_comment_id)
        .await
        .unwrap();
    assert!(
        task_comment_actual
            .updated_at
            .gt(&task_comment_actual.created_at)
    );
    task_comment_expected.created_at = task_comment_actual.created_at; // подменим на валидное явно
    task_comment_expected.updated_at = task_comment_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_comment_expected, task_comment_actual);

    // err: попытаемся удалить
    assert!(db.tbl_task_comments.delete(Uuid::new_v4()).await.is_err());

    // ok: удалим
    assert!(
        db.tbl_task_comments
            .delete(task_comment_actual.task_comment_id)
            .await
            .is_success()
    );

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_task_comments
            .one(task_comment_actual.task_comment_id)
            .await,
        Err(RepositoryError::NotFoundRow)
    );

    // ok: почистим за собой
    db.tbl_tasks.delete(task_id1).await.unwrap();
    db.tbl_teams.delete(team_id).await.unwrap();
    db.tbl_users.delete(user_id1).await.unwrap();
    db.tbl_users.delete(user_id2).await.unwrap();

    // проверим каскадное удаление
    user_id1 = db.tbl_users.create(rand::user()).await.unwrap();
    user_id2 = db.tbl_users.create(rand::user()).await.unwrap();
    team = rand::team();
    team.created_by = user_id1;
    team_id = db.tbl_teams.create(team).await.unwrap();
    task = rand::task();
    task.team_id = team_id;
    task.created_by = user_id1;
    task.assignee_id = None;
    task.status = TaskStatus::Done.to_string();
    task_id1 = db.tbl_tasks.create(task.clone()).await.unwrap();
    task_comment_expected = rand::task_comment();
    task_comment_expected.task_id = task_id1;
    task_comment_expected.user_id = user_id1;
    let task_comment_id1 = db
        .tbl_task_comments
        .create(task_comment_expected)
        .await
        .unwrap();
    task_comment_expected = rand::task_comment();
    task_comment_expected.task_id = task_id1;
    task_comment_expected.user_id = user_id2;
    let task_comment_id2 = db
        .tbl_task_comments
        .create(task_comment_expected)
        .await
        .unwrap();

    db.tbl_users.delete(user_id2).await.unwrap();
    assert_matches!(
        db.tbl_task_comments.one(task_comment_id2).await,
        Err(RepositoryError::NotFoundRow)
    );

    db.tbl_tasks.delete(task_id1).await.unwrap();
    assert_matches!(
        db.tbl_task_comments.one(task_comment_id1).await,
        Err(RepositoryError::NotFoundRow)
    );

    // почистим за собой
    db.tbl_teams.delete(team_id).await.unwrap();
    db.tbl_users.delete(user_id1).await.unwrap();
}
