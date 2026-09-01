#[derive(Clone)]
pub struct Transactor {}

impl Transactor {
    pub fn new() -> Self {
        Self{}
    }
    pub fn begin(&self){}
    pub fn commit(&self){}
    pub fn rollback(&self){}
}