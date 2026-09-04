#[derive(Clone, Default)]
pub struct Transactor {}

impl Transactor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self{}
    }
    #[allow(dead_code)]
    pub fn begin(&self){}
    #[allow(dead_code)]
    pub fn commit(&self){}
    #[allow(dead_code)]
    pub fn rollback(&self){}
}