use argon2::{
    Argon2,
    password_hash::{Error as Argon2Error, PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

pub fn password_hash(str: &str) -> Result<String, Argon2Error> {
    let result = Argon2::default().hash_password(str.as_bytes())?;
    Ok(result.to_string())
}

pub fn password_verify(pass: &str, hash: &str) -> Result<bool, Argon2Error> {
    let password_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(pass.as_ref(), &password_hash)
        .is_ok())
}
