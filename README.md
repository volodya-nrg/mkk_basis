# mkk_basis
ООО МКК "БАЗИС"

## TODO
- авторизацию сделать с применением кук
- наладить grpc (через докер)
- наладить tx
- нужно разобраться с юнит-тестами usecase (моки)
- сделать все относительно интерфейсов
- наладить swagger
- после переписать на grpc-gateway-rust

## Заметки
- установка "cargo install sqlx-cli" для миграций
- миграции: "sqlx migrate add -r foundation" (up, down)
- tokio/axum/tower/hyper/lapin
- tokio - асинхронный runtime
- https://github.com/cksac/fake-rs
- чтоб пропустить тест нужно добавить метку: #[ignore]
- Проблема: OnceCell создает клиент один раз для всех тестов. Но каждый тест запускается в своем
  runtime. Когда первый тест завершается, его runtime может быть уничтожен, но Client (который внутри
  содержит свой runtime) остается в статической переменной.

  Когда второй тест вызывает get_test_server().await, он получает уже существующий клиент, но его
  внутренний runtime уже мог быть уничтожен после завершения первого теста.
- https://ohmycloud.github.io/2025/05/03/authentication-with-axum.html - cookie auth