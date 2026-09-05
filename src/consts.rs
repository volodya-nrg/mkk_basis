pub const MIN_PASSWORD_LEN: usize = 5; // usize из-за count()
pub const ACCESS_TOKEN_NAME: &str = "access_token";
pub const REFRESH_TOKEN_NAME: &str = "refresh_token";
pub const ACCESS_TOKEN_TTL_SEC: i64 = 60 * 20;
pub const REFRESH_TOKEN_TTL_SEC: i64 = (60 * 60 * 24) * 30;
