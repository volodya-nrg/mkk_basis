use chrono::{Duration, Utc};
use jsonwebtoken::errors;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub const TYPE_ACCESS: &str = "access";
pub const TYPE_REFRESH: &str = "refresh";

#[derive(Debug)]
pub enum JWTError {
    ExpiredToken,
    Common(errors::Error),
}
impl fmt::Display for JWTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JWTError::ExpiredToken => write!(f, "token expired"),
            JWTError::Common(s) => write!(f, "{s}"),
        }
    }
}

// From - для e.into() (авто-конвертация)
impl From<errors::Error> for JWTError {
    fn from(value: errors::Error) -> Self {
        match value.kind() {
            errors::ErrorKind::ExpiredSignature => JWTError::ExpiredToken,
            _ => JWTError::Common(value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
    pub role: Option<String>,
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
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        role: Option<String>,
    ) -> Result<String, JWTError> {
        let now = Utc::now();
        let expire = now + Duration::seconds(self.access_expire_secs);
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &AccessClaims {
                sub: user_id,
                exp: expire.timestamp() as usize,
                iat: now.timestamp() as usize,
                token_type: TYPE_ACCESS.to_string(),
                role,
            },
            &jsonwebtoken::EncodingKey::from_secret(self.private_key_bytes.as_slice()),
        )
        .map_err(JWTError::Common)
    }
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, JWTError> {
        let now = Utc::now();
        let expire = now + Duration::seconds(self.refresh_expire_secs);
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &RefreshClaims {
                sub: user_id,
                exp: expire.timestamp() as usize,
                iat: now.timestamp() as usize,
                token_type: TYPE_REFRESH.to_string(),
            },
            &jsonwebtoken::EncodingKey::from_secret(self.private_key_bytes.as_slice()),
        )
        .map_err(JWTError::Common)
    }
    pub fn validate_access_token(&self, token: String) -> Result<AccessClaims, JWTError> {
        jsonwebtoken::decode::<AccessClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.private_key_bytes.as_slice()),
            &self.get_validation(),
        )
        .map(|data| data.claims)
        .map_err(|e| e.into())
    }
    pub fn validate_refresh_token(&self, token: String) -> Result<RefreshClaims, JWTError> {
        jsonwebtoken::decode::<RefreshClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.private_key_bytes.as_slice()),
            &self.get_validation(),
        )
        .map(|data| data.claims)
        .map_err(|e| e.into())
    }
    fn get_validation(&self) -> jsonwebtoken::Validation {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
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
            .generate_access_token(user_id, Some(ROLE_ADMIN.to_string()))
            .unwrap();
        assert!(!access_token.is_empty());

        let refresh_token = jwt.generate_refresh_token(user_id).unwrap();
        assert!(!refresh_token.is_empty());

        let access_claims = jwt.validate_access_token(access_token.clone()).unwrap();
        assert_eq!(TYPE_ACCESS, access_claims.token_type);
        assert_eq!(user_id, access_claims.sub);
        assert!(access_claims.role.is_some());
        assert_eq!(ROLE_ADMIN, access_claims.role.unwrap());

        let refresh_claims = jwt.validate_refresh_token(refresh_token.clone()).unwrap();
        assert_eq!(TYPE_REFRESH, refresh_claims.token_type);
        assert_eq!(user_id, refresh_claims.sub);

        // задержимся чтоб время прошло
        sleep(std::time::Duration::from_secs(SEC + SEC));

        assert_matches!(
            jwt.validate_access_token(access_token.clone()),
            Err(JWTError::ExpiredToken),
        );
        assert_matches!(
            jwt.validate_refresh_token(refresh_token.clone()),
            Err(JWTError::ExpiredToken),
        );
    }
}
