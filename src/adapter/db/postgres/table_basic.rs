#[derive(Clone)] // из-за axum-state
pub struct TableBasic {
    pub name: String,
    pub fields: Vec<String>,
}