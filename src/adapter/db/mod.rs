pub mod errors;
pub mod models;
pub mod postgres;

// приватный модуль только для внутреннего использования
mod internal {
    pub trait Table {
        fn get_name(&self) -> &str {
            ""
        }
        fn get_fields(&self) -> &[&str] {
            &[]
        }
    }
}
