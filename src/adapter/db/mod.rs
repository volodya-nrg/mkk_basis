pub mod errors;
pub mod models;
pub mod postgres;
pub mod storage;

// приватный модуль только для внутреннего использования
mod internal {
    pub trait Table {
        fn get_table_name(&self) -> &str {
            ""
        }
        fn get_fields(&self) -> &[&str] {
            &[]
        }
    }
}
