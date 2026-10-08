// трейт необходим для каждой таблицы, чтоб соблюдался единый контракт
pub trait Table {
    fn get_table_name(&self) -> &str {
        ""
    }
    fn get_fields(&self) -> &[&str] {
        &[]
    }
}
