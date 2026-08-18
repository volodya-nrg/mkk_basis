use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{
    Error as Argon2Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};

pub fn password_hash(str: &str) -> Result<String, Argon2Error> {
    let salt = SaltString::generate(&mut OsRng);
    let result = Argon2::default().hash_password(str.as_bytes(), &salt)?;
    Ok(result.to_string())
}
pub fn password_verify(pass: &str, hash: &str) -> Result<bool, Argon2Error> {
    let password_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(pass.as_ref(), &password_hash)
        .is_ok())
}
