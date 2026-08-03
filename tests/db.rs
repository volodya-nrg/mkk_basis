mod transport;
mod common;

use chrono::Local;
use fake::{Fake, Faker};
use mkk_basis::adapter::db::errors::Error;
use mkk_basis::adapter::db::models::*;
use mkk_basis::adapter::db::postgres::Postgres;
use mkk_basis::adapter::db::postgres::tables::tasks::Status as TaskStatus;
use sqlx::postgres::PgPoolOptions;
use std::assert_matches;
use uuid::Uuid;

const DSN: &str =
    "postgres://postgres:postgres@127.0.0.1:5432/postgres?search_path=mkk_basis&sslmode=disable";

#[tokio::test]
async fn check_users() {
    let pool = PgPoolOptions::new().connect(DSN).await.unwrap();
    let db = Postgres::new(&pool);
    let time_now = Local::now();

    // err: проверим что запись не находит
    assert_matches!(db.tbl_users.one(Uuid::new_v4()).await, Err(Error::NotFound));

    // ok: проверим что запись создается
    let mut user_expected: User = Faker.fake();
    let result = db.tbl_users.create(user_expected.clone()).await;
    assert!(result.is_ok());
    user_expected.user_id = result.unwrap(); // подменим на валидное явно

    // err: проверим что нельзя добавить такую же запись
    assert!(db.tbl_users.create(user_expected.clone()).await.is_err());

    // ok: проверим что запись можно получить и данные их равны
    let result = db.tbl_users.one(user_expected.user_id).await;
    assert!(result.is_ok());
    let user_actual = result.unwrap();
    user_expected.created_at = user_actual.created_at; // подменим на валидное явно
    user_expected.updated_at = user_actual.updated_at; // подменим на валидное явно
    assert_eq!(user_expected, user_actual);
    assert!(user_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let result = db.tbl_users.list(-1, -1).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let result = db.tbl_users.list(0, 0).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим
    assert!(db.tbl_users.update(Faker.fake::<User>()).await.is_err());

    // ok: изменим и проверим пользователя
    let mut user_expected: User = Faker.fake();
    user_expected.user_id = user_actual.user_id; // подменим на валидное явно
    assert!(db.tbl_users.update(user_expected.clone()).await.is_ok());
    let result = db.tbl_users.one(user_expected.user_id).await;
    assert!(result.is_ok());
    let user_actual = result.unwrap();
    user_expected.created_at = user_actual.created_at; // подменим на валидное явно
    user_expected.updated_at = user_actual.updated_at; // подменим на валидное явно
    assert_eq!(user_expected, user_actual);
    assert!(user_actual.updated_at.gt(&user_actual.created_at));

    // err: удалим не известного пользователя
    assert!(db.tbl_users.delete(Uuid::new_v4()).await.is_err());

    // ok: удалим пользователя
    assert!(db.tbl_users.delete(user_actual.user_id).await.is_ok());

    // ok: не нашли пользователя, как и задумано
    assert_matches!(
        db.tbl_users.one(user_actual.user_id).await,
        Err(Error::NotFound)
    );
}

#[tokio::test]
async fn check_teams() {
    let pool = PgPoolOptions::new().connect(DSN).await.unwrap();
    let db = Postgres::new(&pool);
    let time_now = Local::now();

    // ok: создадим пользователя
    let user_id: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();

    // err: проверим что команду не находит
    assert_matches!(db.tbl_teams.one(Uuid::new_v4()).await, Err(Error::NotFound));

    // err: попытаемся создать команду, но такой нет
    assert!(db.tbl_teams.create(Faker.fake::<Team>()).await.is_err());

    // ok: проверим что создается
    let mut team_expected: Team = Faker.fake();
    team_expected.created_by = user_id; // зависит от user-а
    let result = db.tbl_teams.create(team_expected.clone()).await;
    assert!(result.is_ok());
    team_expected.team_id = result.unwrap(); // подменим на валидное явно

    // err: проверим что нельзя добавить такую же запись
    assert!(db.tbl_teams.create(team_expected.clone()).await.is_err());

    // ok: проверим что можно получить и их данные равны
    let result = db.tbl_teams.one(team_expected.team_id).await;
    assert!(result.is_ok());
    let team_actual = result.unwrap();
    team_expected.created_at = team_actual.created_at; // подменим на валидное явно
    team_expected.updated_at = team_actual.updated_at; // подменим на валидное явно
    assert_eq!(team_expected, team_actual);
    assert!(team_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let result = db.tbl_teams.list(-1, -1).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let result = db.tbl_teams.list(0, 0).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим неизвестного
    assert!(db.tbl_teams.update(Faker.fake::<Team>()).await.is_err());

    // ok: изменим и проверим
    let mut team_expected: Team = Faker.fake();
    team_expected.team_id = team_actual.team_id; // подменим на валидное явно
    team_expected.created_by = user_id;
    assert!(db.tbl_teams.update(team_expected.clone()).await.is_ok());
    let result = db.tbl_teams.one(team_expected.team_id).await;
    assert!(result.is_ok());
    let team_actual = result.unwrap();
    team_expected.created_at = team_actual.created_at; // подменим на валидное явно
    team_expected.updated_at = team_actual.updated_at; // подменим на валидное явно
    assert_eq!(team_expected, team_actual);
    assert!(team_actual.updated_at.gt(&team_actual.created_at));

    // err: попытаемся удалить
    assert!(db.tbl_teams.delete(Uuid::new_v4()).await.is_err());

    // err: нельзя удалить пользователя пока есть привязанная команда
    assert!(db.tbl_users.delete(user_id).await.is_err());

    // ok: удалим
    assert!(db.tbl_teams.delete(team_actual.team_id).await.is_ok());

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_teams.one(team_actual.team_id).await,
        Err(Error::NotFound)
    );

    // ok: почистим за собой
    assert!(db.tbl_users.delete(user_id).await.is_ok());
}

#[tokio::test]
async fn check_team_members() {
    let pool = PgPoolOptions::new().connect(DSN).await.unwrap();
    let db = Postgres::new(&pool);
    let time_now = Local::now();

    // ok: создадим команду и пользователя
    let user_id: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut team: Team = Faker.fake();
    team.created_by = user_id;
    let team_id: Uuid = db.tbl_teams.create(team).await.unwrap();

    // err: проверим что не находит
    assert_matches!(
        db.tbl_team_members
            .one(Uuid::new_v4(), Uuid::new_v4())
            .await,
        Err(Error::NotFound)
    );

    // err: попытаемся создать, но связанных данных нет
    assert!(
        db.tbl_team_members
            .create(Faker.fake::<TeamMember>())
            .await
            .is_err()
    );

    // ok: проверим что создается
    let mut team_member_expected: TeamMember = Faker.fake();
    team_member_expected.team_id = team_id; // зависит от team-а
    team_member_expected.user_id = user_id; // зависит от user-а
    assert!(
        db.tbl_team_members
            .create(team_member_expected.clone())
            .await
            .is_ok()
    );

    // err: проверим что нельзя добавить такую же запись
    assert!(
        db.tbl_team_members
            .create(team_member_expected.clone())
            .await
            .is_err()
    );

    // ok: проверим что можно получить и их данные равны
    let result = db
        .tbl_team_members
        .one(team_member_expected.team_id, team_member_expected.user_id)
        .await;
    assert!(result.is_ok());
    let team_member_actual = result.unwrap();
    team_member_expected.created_at = team_member_actual.created_at; // подменим на валидное явно
    assert_eq!(team_member_expected, team_member_actual);
    assert!(team_member_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let result = db.tbl_team_members.all().await;
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());

    // err: попытаемся удалить
    assert!(
        db.tbl_team_members
            .delete(Uuid::new_v4(), Uuid::new_v4())
            .await
            .is_err()
    );

    // ok: удалим
    assert!(
        db.tbl_team_members
            .delete(team_member_actual.team_id, team_member_actual.user_id)
            .await
            .is_ok()
    );

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_team_members
            .one(team_member_actual.team_id, team_member_actual.user_id)
            .await,
        Err(Error::NotFound)
    );

    // ok: почистим за собой
    assert!(db.tbl_teams.delete(team_id).await.is_ok());
    assert!(db.tbl_users.delete(user_id).await.is_ok());

    // проверим каскадное удаление, относительно team_id
    let user_id: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut team: Team = Faker.fake();
    team.created_by = user_id;
    let team_id: Uuid = db.tbl_teams.create(team).await.unwrap();
    let mut team_member_expected: TeamMember = Faker.fake();
    team_member_expected.team_id = team_id;
    team_member_expected.user_id = user_id;
    assert!(
        db.tbl_team_members
            .create(team_member_expected.clone())
            .await
            .is_ok()
    );
    assert!(db.tbl_teams.delete(team_id).await.is_ok());
    assert_matches!(
        db.tbl_team_members
            .one(team_member_expected.team_id, team_member_expected.user_id)
            .await,
        Err(Error::NotFound)
    );
    assert!(db.tbl_users.delete(user_id).await.is_ok());
}

#[tokio::test]
async fn check_tasks() {
    let pool = PgPoolOptions::new().connect(DSN).await.unwrap();
    let db = Postgres::new(&pool);
    let time_now = Local::now();

    // ok: создадим пользователя и команду
    let user_id: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut team: Team = Faker.fake();
    team.created_by = user_id;
    let team_id: Uuid = db.tbl_teams.create(team).await.unwrap();

    // err: проверим что задачу не находит
    assert_matches!(db.tbl_tasks.one(Uuid::new_v4()).await, Err(Error::NotFound));

    // err: попытаемся создать задачу, но такой нет
    assert!(db.tbl_tasks.create(Faker.fake::<Task>()).await.is_err());

    // ok: проверим что создается
    let mut task_expected: Task = Faker.fake();
    task_expected.team_id = team_id; // зависит от team
    task_expected.created_by = user_id; // зависит от user-а
    task_expected.assignee_id = Some(user_id); // зависит от user-а
    task_expected.status = format!("{:?}", TaskStatus::Start).to_lowercase();
    let result = db.tbl_tasks.create(task_expected.clone()).await;
    assert!(result.is_ok());
    task_expected.task_id = result.unwrap(); // подменим на валидное явно

    // err: проверим что нельзя добавить такую же запись
    assert!(db.tbl_tasks.create(task_expected.clone()).await.is_err());

    // ok: проверим что можно получить и их данные равны
    let result = db.tbl_tasks.one(task_expected.task_id).await;
    assert!(result.is_ok());
    let task_actual = result.unwrap();
    task_expected.created_at = task_actual.created_at; // подменим на валидное явно
    task_expected.updated_at = task_actual.updated_at; // подменим на валидное явно
    assert_eq!(task_expected, task_actual);
    assert!(task_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let result = db.tbl_tasks.list(-1, -1).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let result = db.tbl_tasks.list(0, 0).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим неизвестного
    assert!(db.tbl_tasks.update(Faker.fake::<Task>()).await.is_err());

    // ok: изменим и проверим
    let mut task_expected: Task = Faker.fake();
    task_expected.task_id = task_actual.task_id;
    task_expected.team_id = team_id; // зависит от team
    task_expected.created_by = user_id; // зависит от user-а
    task_expected.assignee_id = None; // зависит от user-а
    task_expected.status = format!("{:?}", TaskStatus::Cancelled).to_lowercase();
    let result = db.tbl_tasks.update(task_expected.clone()).await;
    assert!(result.is_ok());
    let result = db.tbl_tasks.one(task_expected.task_id).await;
    assert!(result.is_ok());
    let task_actual = result.unwrap();
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
    assert!(db.tbl_tasks.delete(task_actual.task_id).await.is_ok());

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_tasks.one(task_actual.task_id).await,
        Err(Error::NotFound)
    );

    // ok: почистим за собой
    assert!(db.tbl_teams.delete(task_actual.team_id).await.is_ok());
    assert!(db.tbl_users.delete(user_id).await.is_ok());
}

#[tokio::test]
async fn check_task_histories() {
    let pool = PgPoolOptions::new().connect(DSN).await.unwrap();
    let db = Postgres::new(&pool);
    let time_now = Local::now();

    // ok: создадим зависимости
    let user_id: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let user_id2: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut team: Team = Faker.fake();
    team.created_by = user_id;
    let team_id: Uuid = db.tbl_teams.create(team).await.unwrap();
    let mut task: Task = Faker.fake();
    task.team_id = team_id;
    task.created_by = user_id;
    task.assignee_id = None;
    task.status = format!("{:?}", TaskStatus::Todo).to_lowercase();
    let task_id = db.tbl_tasks.create(task).await.unwrap();

    // err: проверим что не находит
    assert_matches!(
        db.tbl_task_histories.one(Uuid::new_v4()).await,
        Err(Error::NotFound)
    );

    // err: попытаемся создать
    assert!(
        db.tbl_task_histories
            .create(Faker.fake::<TaskHistory>())
            .await
            .is_err()
    );

    // ok: проверим что создается
    let mut task_history_expected: TaskHistory = Faker.fake();
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id;
    let result = db
        .tbl_task_histories
        .create(task_history_expected.clone())
        .await;
    assert!(result.is_ok());
    task_history_expected.task_history_id = result.unwrap();

    // err: проверим что нельзя добавить такую же запись
    assert!(
        db.tbl_task_histories
            .create(task_history_expected.clone())
            .await
            .is_err()
    );

    // ok: проверим что можно получить и их данные равны
    let result = db
        .tbl_task_histories
        .one(task_history_expected.task_history_id)
        .await;
    assert!(result.is_ok());
    let task_history_actual = result.unwrap();
    task_history_expected.created_at = task_history_actual.created_at; // подменим на валидное явно
    assert_eq!(task_history_expected, task_history_actual);
    assert!(task_history_expected.created_at.gt(&time_now));

    // ok: проверим что список не пустой
    let result = db.tbl_task_histories.list(-1, -1).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let result = db.tbl_task_histories.list(0, 0).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим неизвестного
    assert!(
        db.tbl_task_histories
            .update(Faker.fake::<TaskHistory>())
            .await
            .is_err()
    );

    // ok: изменим и проверим
    let mut task_history_expected: TaskHistory = Faker.fake();
    task_history_expected.task_history_id = task_history_actual.task_history_id;
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id2;

    let result = db
        .tbl_task_histories
        .update(task_history_expected.clone())
        .await;
    assert!(result.is_ok());
    let result = db
        .tbl_task_histories
        .one(task_history_expected.task_history_id)
        .await;
    assert!(result.is_ok());
    let task_history_actual = result.unwrap();
    task_history_expected.created_at = task_history_actual.created_at; // подменим на валидное явно
    assert_eq!(task_history_expected, task_history_actual);

    // err: попытаемся удалить
    assert!(db.tbl_task_histories.delete(Uuid::new_v4()).await.is_err());

    // ok: удалим
    assert!(
        db.tbl_task_histories
            .delete(task_history_actual.task_history_id)
            .await
            .is_ok()
    );

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_task_histories
            .one(task_history_actual.task_history_id)
            .await,
        Err(Error::NotFound)
    );

    // ok: почистим за собой
    assert!(db.tbl_tasks.delete(task_id).await.is_ok());
    assert!(db.tbl_teams.delete(team_id).await.is_ok());
    assert!(db.tbl_users.delete(user_id).await.is_ok());
    assert!(db.tbl_users.delete(user_id2).await.is_ok());

    // проверим каскадное удаление относительно task_id
    let user_id: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut team: Team = Faker.fake();
    team.created_by = user_id;
    let team_id: Uuid = db.tbl_teams.create(team).await.unwrap();
    let mut task: Task = Faker.fake();
    task.team_id = team_id;
    task.created_by = user_id;
    task.assignee_id = None;
    task.status = format!("{:?}", TaskStatus::Todo).to_lowercase();
    let task_id = db.tbl_tasks.create(task.clone()).await.unwrap();

    let mut task_history_expected: TaskHistory = Faker.fake();
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id;
    let task_history_id = db
        .tbl_task_histories
        .create(task_history_expected)
        .await
        .unwrap();
    db.tbl_tasks.delete(task_id).await.unwrap();
    assert_matches!(
        db.tbl_task_histories.one(task_history_id).await,
        Err(Error::NotFound)
    );

    // проверим каскадное удаление относительно user_id2
    let task_id = db.tbl_tasks.create(task).await.unwrap();
    let user_id2: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut task_history_expected: TaskHistory = Faker.fake();
    task_history_expected.task_id = task_id;
    task_history_expected.user_id = user_id2;
    let task_history_id = db
        .tbl_task_histories
        .create(task_history_expected)
        .await
        .unwrap();
    db.tbl_users.delete(user_id2).await.unwrap();
    assert_matches!(
        db.tbl_task_histories.one(task_history_id).await,
        Err(Error::NotFound)
    );

    // ok: почистим за собой
    assert!(db.tbl_tasks.delete(task_id).await.is_ok());
    assert!(db.tbl_teams.delete(team_id).await.is_ok());
    assert!(db.tbl_users.delete(user_id).await.is_ok());
}

#[tokio::test]
async fn check_task_comments() {
    let pool = PgPoolOptions::new().connect(DSN).await.unwrap();
    let db = Postgres::new(&pool);
    let time_now = Local::now();

    // ok: создадим зависимости
    let user_id: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let user_id2: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut team: Team = Faker.fake();
    team.created_by = user_id;
    let team_id: Uuid = db.tbl_teams.create(team).await.unwrap();
    let mut task: Task = Faker.fake();
    task.team_id = team_id;
    task.created_by = user_id;
    task.assignee_id = None;
    task.status = format!("{:?}", TaskStatus::Todo).to_lowercase();
    let task_id = db.tbl_tasks.create(task).await.unwrap();

    // err: проверим что не находит
    assert_matches!(
        db.tbl_task_comments.one(Uuid::new_v4()).await,
        Err(Error::NotFound)
    );

    // err: попытаемся создать, отсутствуют зависимости
    assert!(
        db.tbl_task_comments
            .create(Faker.fake::<TaskComment>())
            .await
            .is_err()
    );

    // ok: проверим что создается
    let mut task_comment_expected: TaskComment = Faker.fake();
    task_comment_expected.task_id = task_id;
    task_comment_expected.user_id = user_id;
    let result = db
        .tbl_task_comments
        .create(task_comment_expected.clone())
        .await;
    assert!(result.is_ok());
    task_comment_expected.task_comment_id = result.unwrap();

    // ok: проверим что можно добавить такую же запись
    assert!(
        db.tbl_task_comments
            .create(task_comment_expected.clone())
            .await
            .is_ok()
    );

    // ok: проверим что можно получить и их данные равны
    let result = db
        .tbl_task_comments
        .one(task_comment_expected.task_comment_id)
        .await;
    assert!(result.is_ok());
    let task_comment_actual = result.unwrap();
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
    let result = db.tbl_task_comments.list(-1, -1).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(!items.is_empty());
    assert!(total > 0);

    // ok: проверим пустой результат
    let result = db.tbl_task_comments.list(0, 0).await;
    assert!(result.is_ok());
    let (items, total) = result.unwrap();
    assert!(items.is_empty());
    assert!(total > 0); // список пустой, но общее кол-во есть

    // err: изменим неизвестного
    assert!(
        db.tbl_task_comments
            .update(Faker.fake::<TaskComment>())
            .await
            .is_err()
    );

    // ok: изменим и проверим
    let mut task_comment_expected: TaskComment = Faker.fake();
    task_comment_expected.task_comment_id = task_comment_actual.task_comment_id;
    task_comment_expected.task_id = task_id;
    task_comment_expected.user_id = user_id2;

    let result = db
        .tbl_task_comments
        .update(task_comment_expected.clone())
        .await;
    assert!(result.is_ok());
    let result = db
        .tbl_task_comments
        .one(task_comment_expected.task_comment_id)
        .await;
    assert!(result.is_ok());
    let task_comment_actual = result.unwrap();
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
    assert!(
        db.tbl_task_comments
            .delete(task_comment_actual.task_comment_id)
            .await
            .is_ok()
    );

    // ok: не нашли, как и задумано
    assert_matches!(
        db.tbl_task_comments
            .one(task_comment_actual.task_comment_id)
            .await,
        Err(Error::NotFound)
    );

    // ok: почистим за собой
    assert!(db.tbl_tasks.delete(task_id).await.is_ok());
    assert!(db.tbl_teams.delete(team_id).await.is_ok());
    assert!(db.tbl_users.delete(user_id).await.is_ok());
    assert!(db.tbl_users.delete(user_id2).await.is_ok());

    // проверим каскадное удаление относительно task_id
    let user_id: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut team: Team = Faker.fake();
    team.created_by = user_id;
    let team_id: Uuid = db.tbl_teams.create(team).await.unwrap();
    let mut task: Task = Faker.fake();
    task.team_id = team_id;
    task.created_by = user_id;
    task.assignee_id = None;
    task.status = format!("{:?}", TaskStatus::Done).to_lowercase();
    let task_id = db.tbl_tasks.create(task.clone()).await.unwrap();

    let mut task_comment_expected: TaskComment = Faker.fake();
    task_comment_expected.task_id = task_id;
    task_comment_expected.user_id = user_id;
    let task_comment_id = db
        .tbl_task_comments
        .create(task_comment_expected)
        .await
        .unwrap();
    db.tbl_tasks.delete(task_id).await.unwrap();
    assert_matches!(
        db.tbl_task_comments.one(task_comment_id).await,
        Err(Error::NotFound)
    );

    // проверим каскадное удаление относительно user_id2
    let task_id = db.tbl_tasks.create(task).await.unwrap();
    let user_id2: Uuid = db.tbl_users.create(Faker.fake::<User>()).await.unwrap();
    let mut task_comment_expected: TaskComment = Faker.fake();
    task_comment_expected.task_id = task_id;
    task_comment_expected.user_id = user_id2;
    let task_comment_id = db
        .tbl_task_comments
        .create(task_comment_expected)
        .await
        .unwrap();
    db.tbl_users.delete(user_id2).await.unwrap();
    assert_matches!(
        db.tbl_task_comments.one(task_comment_id).await,
        Err(Error::NotFound)
    );

    // ok: почистим за собой
    assert!(db.tbl_tasks.delete(task_id).await.is_ok());
    assert!(db.tbl_teams.delete(team_id).await.is_ok());
    assert!(db.tbl_users.delete(user_id).await.is_ok());
}
