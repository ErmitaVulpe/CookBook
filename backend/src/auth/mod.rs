pub mod jwt;

use argon2::password_hash::{ rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString };
use lazy_static::lazy_static;

lazy_static! {
    static ref ARGON2_CONF: argon2::Argon2<'static> = {
        argon2::Argon2::default()
    };
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    ARGON2_CONF.hash_password(password.as_bytes(), &salt).unwrap().to_string()
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hashed_password).unwrap();
    ARGON2_CONF.verify_password(password.as_bytes(), &parsed_hash).is_ok()
}


#[derive(Debug, Clone, PartialEq)]
pub enum CookieName {
    RefreshToken,
    AccessToken,
}

impl std::fmt::Display for CookieName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookieName::RefreshToken => write!(f, "refresh_token"),
            CookieName::AccessToken => write!(f, "access_token"),
        }
    }
}
