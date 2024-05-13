use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Serialize, Deserialize};
use serde_repr::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: u64,
    pub permissions: Permissions,
}

impl Claims {
    pub fn new(permissions: Permissions) -> Self {
        Self {
            exp: chrono::offset::Utc::now().timestamp() as u64 + (6 * 60 * 60),
            permissions,
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
pub enum Permissions {
    Admin,
}

pub struct JwtConfig {
    header: Header,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtConfig {
    pub fn new() -> Self {
        use rand::prelude::*;

        let mut secret = [0u8; 64];
        let mut rng = rand::thread_rng();
        rng.fill(&mut secret);

        Self {
            header: Header::default(),
            encoding_key: EncodingKey::from_secret(&secret),
            decoding_key: DecodingKey::from_secret(&secret),
        }
    }

    pub fn generate(&self, claims: Claims) -> Result<String, Error> {
        encode(&self.header, &claims, &self.encoding_key)
    }

    pub fn decode(&self, token: &str) -> Result<Claims, Error> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|x| x.claims)
    }
}
