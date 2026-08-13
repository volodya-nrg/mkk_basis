use chrono::{Duration, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
    errors::ErrorKind as JWTErrorKind,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error as ThisError;

const TYPE_ACCESS: &str = "access";
const TYPE_REFRESH: &str = "refresh";

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("expired token")]
    ExpiredToken,
    #[error("{0}")]
    Common(String),
}
impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.kind() {
            JWTErrorKind::ExpiredSignature => Error::ExpiredToken,
            _ => Error::Common(value.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
}

#[derive(Clone)] // из-за usecase Auth
pub struct Jwt {
    private_key_bytes: Vec<u8>,
    access_expire_secs: i64,
    refresh_expire_secs: i64,
}
impl Jwt {
    pub fn new(
        private_key_bytes: Vec<u8>,
        access_expire_secs: i64,
        refresh_expire_secs: i64,
    ) -> Self {
        Self {
            private_key_bytes,
            access_expire_secs,
            refresh_expire_secs,
        }
    }
    pub fn generate_access_token(&self, user_id: Uuid, role: String) -> Result<String, Error> {
        let now = Utc::now();
        let expire = now + Duration::seconds(self.access_expire_secs);
        encode(
            &Header::default(),
            &AccessClaims {
                sub: user_id,
                exp: expire.timestamp() as usize,
                iat: now.timestamp() as usize,
                token_type: TYPE_ACCESS.to_string(),
                role,
            },
            &EncodingKey::from_secret(self.private_key_bytes.as_slice()),
        )
        .map_err(|e| Error::Common(e.to_string()))
    }
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, Error> {
        let now = Utc::now();
        let expire = now + Duration::seconds(self.refresh_expire_secs);
        encode(
            &Header::default(),
            &RefreshClaims {
                sub: user_id,
                exp: expire.timestamp() as usize,
                iat: now.timestamp() as usize,
                token_type: TYPE_REFRESH.to_string(),
            },
            &EncodingKey::from_secret(self.private_key_bytes.as_slice()),
        )
        .map_err(|e| Error::Common(e.to_string()))
    }
    pub fn validate_access_token(&self, token: String) -> Result<AccessClaims, Error> {
        decode::<AccessClaims>(
            token,
            &DecodingKey::from_secret(self.private_key_bytes.as_slice()),
            &self.get_validation(),
        )
        .map(|data| data.claims)
        .map_err(|e| e.into())
    }
    #[allow(dead_code)]
    pub fn validate_refresh_token(&self, token: String) -> Result<RefreshClaims, Error> {
        decode::<RefreshClaims>(
            token,
            &DecodingKey::from_secret(self.private_key_bytes.as_slice()),
            &self.get_validation(),
        )
        .map(|data| data.claims)
        .map_err(|e| e.into())
    }
    fn get_validation(&self) -> Validation {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0; // разница во времени
        validation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::assert_matches;
    use std::thread::sleep;

    fn generate_private_key_bytes(len: usize) -> Vec<u8> {
        let mut key = vec![0u8; len];
        rand::rng().fill_bytes(&mut key);
        key
    }

    #[test]
    fn check_all() {
        const SEC: u64 = 1;
        const ROLE_ADMIN: &str = "admin";

        let private_key = generate_private_key_bytes(32);
        let jwt = Jwt::new(private_key, SEC as i64, SEC as i64);
        let user_id = Uuid::new_v4();

        let access_token = jwt
            .generate_access_token(user_id, ROLE_ADMIN.to_string())
            .unwrap();
        assert!(!access_token.is_empty());

        let refresh_token = jwt.generate_refresh_token(user_id).unwrap();
        assert!(!refresh_token.is_empty());

        let access_claims = jwt.validate_access_token(access_token.clone()).unwrap();
        assert_eq!(TYPE_ACCESS, access_claims.token_type);
        assert_eq!(user_id, access_claims.sub);
        assert_eq!(ROLE_ADMIN, access_claims.role);

        let refresh_claims = jwt.validate_refresh_token(refresh_token.clone()).unwrap();
        assert_eq!(TYPE_REFRESH, refresh_claims.token_type);
        assert_eq!(user_id, refresh_claims.sub);

        // задержимся чтоб время прошло
        sleep(std::time::Duration::from_secs(SEC + SEC));

        assert_matches!(
            jwt.validate_access_token(access_token.clone()),
            Err(Error::ExpiredToken),
        );
        assert_matches!(
            jwt.validate_refresh_token(refresh_token.clone()),
            Err(Error::ExpiredToken),
        );
    }
}
