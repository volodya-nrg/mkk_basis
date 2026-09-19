#![cfg_attr(not(test), deny(clippy::unwrap_used))] // Запрещает использование .unwrap() на Option и Result
#![cfg_attr(not(test), deny(clippy::expect_used))] // Запрещает .expect("...")
#![cfg_attr(not(test), deny(clippy::panic))] // Запрещает panic!(), unreachable!(), t-odo!(), unimplemented!() и тд
#![cfg_attr(not(test), deny(unused_must_use))] // Запрещает игнорировать значения, помеченные #[must_use]

// Это библиотечный связующий файл. Через него предоставляется доступ ко внутренностям главной программы
// для например /src/bin и /tests.

pub mod adapter;
pub mod consts;
pub mod err_msg;
pub mod transport;
pub mod usecase;
