pub trait NameAndFields {
    fn get_name(&self) -> &str;
    fn get_fields(&self) -> &[&str];
}
