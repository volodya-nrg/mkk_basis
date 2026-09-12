use uuid::Uuid;

// подгрузим сюда все grpc-модели
include!(concat!(env!("OUT_DIR"), "/mkk_basis_service.v1.rs"));

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: Option<String>,
}
