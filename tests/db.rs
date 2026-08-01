use fake::{Fake, Faker};
use mkk_basis::adapter::db::errors::Error;
use mkk_basis::adapter::db::models::User;
use mkk_basis::adapter::db::postgres::Postgres;
use sqlx::postgres::PgPoolOptions;
use std::assert_matches;
use uuid::Uuid;

const DSN: &str =
    "postgres://postgres:postgres@127.0.0.1:5432/postgres?search_path=mkk_basis&sslmode=disable";

#[tokio::test]
async fn check_users() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DSN)
        .await
        .unwrap();
    let db = Postgres::new(&pool);

    // err: проверим что пользователя не находит
    assert_matches!(db.tbl_users.one(Uuid::new_v4()).await, Err(Error::NotFound));

    // ok: проверим что пользователь создается
    let mut user_expected: User = Faker.fake();
    let result = db.tbl_users.create(user_expected.clone()).await;
    assert!(result.is_ok());
    user_expected.user_id = result.unwrap(); // подменим на валидное явно

    // err: проверим что нельзя добавить такого же пользователя, с тем же е-мэйлом
    assert!(db.tbl_users.create(user_expected.clone()).await.is_err());

    // ok: проверим что пользователя можно получить и их данные равны
    let result = db.tbl_users.one(user_expected.user_id).await;
    assert!(result.is_ok());
    let user_actual = result.unwrap();
    user_expected.created_at = user_actual.created_at; // подменим на валидное явно
    user_expected.updated_at = user_actual.updated_at; // подменим на валидное явно
    assert_eq!(user_expected, user_actual);

    // ok: проверим что список пользователей не пустой
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

    // err: изменим неизвестного пользователя
    let result = db.tbl_users.update(Faker.fake()).await;
    assert!(result.is_err());

    // ok: изменим и проверим пользователя
    let mut user_expected: User = Faker.fake();
    user_expected.user_id = user_actual.user_id; // подменим на валидное явно
    let result = db.tbl_users.update(user_expected.clone()).await;
    assert!(result.is_ok());
    let result = db.tbl_users.one(user_expected.user_id).await;
    assert!(result.is_ok());
    let user_actual = result.unwrap();
    assert_ne!(user_expected.created_at, user_actual.created_at);
    assert_ne!(user_expected.updated_at, user_actual.updated_at);
    user_expected.created_at = user_actual.created_at; // подменим на валидное явно
    user_expected.updated_at = user_actual.updated_at; // подменим на валидное явно
    assert_eq!(user_expected, user_actual);

    // err: удалим не известного пользователя
    let result = db.tbl_users.delete(Uuid::new_v4()).await;
    assert!(result.is_err());

    // ok: удалим пользователя
    assert!(db.tbl_users.delete(user_actual.user_id).await.is_ok());

    // ok: не нашли пользователя, как и задумано
    assert_matches!(
        db.tbl_users.one(user_actual.user_id).await,
        Err(Error::NotFound)
    );
}
