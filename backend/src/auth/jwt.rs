use crate::unwrap_pretty::UnwrapPretty;
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use chrono::{Duration, Utc};

pub fn new(jwt_secret: &str) -> JwtConfig {
    JwtConfig::init(jwt_secret)
}

#[derive(Clone)]
pub struct JwtConfig {
    header: Header,
    validation: Validation,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    valid_duration: Duration,
}

impl std::fmt::Debug for JwtConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JwtConfig {{ ... }}")
    }
}

impl JwtConfig {
    pub fn init(jwt_secret: &str) -> Self {
        let validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.required_spec_claims = std::collections::HashSet::with_capacity(0);
            validation
        };

        JwtConfig {
            header: Header::new(Algorithm::HS256),
            validation,
            encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
            valid_duration: Duration::hours(5),
        }
    }

    pub fn encoding_secret(mut self, jwt_secret: &str) -> Self {
        self.encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        self
    }

    pub fn decoding_secret(mut self, jwt_secret: &str) -> Self {
        self.decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        self
    }

    /// Creates a new jwt struct. The times will be calculated automatically
    pub fn new(
        &self,
        jwt_type: JwtType,
        username: &str,
    ) -> JwtDeserialized {
        let issuing = Utc::now();
        let expiration = issuing + self.valid_duration;
        JwtDeserialized::new(
            jwt_type,
            username,
            &issuing,
            &expiration
        )
    }

    pub fn from_str(
        &self,
        jwt_str: String,
    ) -> JwtSerialized {
        JwtSerialized::from(jwt_str)
    }

    pub fn serilize(&self, jwt: JwtDeserialized) -> JwtSerialized {
        jwt.serialize(self)
    }

    pub fn derilize_str(&self, jwt_str: String,) -> Result<JwtDeserialized, jsonwebtoken::errors::Error> {
        self
            .from_str(jwt_str)
            .deserialize(self)
    }

    pub fn deserialize(&self, jwt: JwtSerialized) -> Result<JwtDeserialized, jsonwebtoken::errors::Error> {
        jwt.deserialize(self)
    }
}


#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum JwtType {
    #[serde(rename = "access_token")] AccessToken,
    #[serde(rename = "refresh_token")] RefreshToken,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct JwtDeserialized {
    jwt_type: JwtType,
    username: String,
    issuing: DateTime<Utc>,
    expiration: DateTime<Utc>,
}

impl JwtDeserialized {
    fn new(
        jwt_type: JwtType,
        username: &str,
        issuing: &DateTime<Utc>,
        expiration: &DateTime<Utc>,
    ) -> Self {
        JwtDeserialized {
            jwt_type: jwt_type,
            username: String::from(username),
            issuing: issuing.clone(),
            expiration: expiration.clone(),
        }
    }

    pub fn serialize(self, config: &JwtConfig) -> JwtSerialized {
        let encoded_string = encode(
            &config.header, 
            &self, &config.encoding_key
        ).unwrap_pretty("Encountered an unexpected error when serializing jwt");
        JwtSerialized::from(encoded_string)
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_issuing(&self) -> DateTime<Utc> {
        self.issuing
    }

    pub fn get_expiration(&self) -> DateTime<Utc> {
        self.expiration
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct JwtSerialized {
    value: String,
}

impl From<String> for JwtSerialized {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl std::fmt::Display for JwtSerialized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl JwtSerialized {
    pub fn deserialize(self, config: &JwtConfig) -> Result<JwtDeserialized, jsonwebtoken::errors::Error> {
        decode::<JwtDeserialized>(
            &self.to_string(), 
            &config.decoding_key, 
            &config.validation
        )
            .map(|data| data.claims)
    }
}




#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    mod jwt { // Reexport so i can use this module as i would normally
        pub use super::super::*;
    }

    
    #[test]
    fn serializing_and_deserializing() {
        let jwt_secret = "Secret string";
        let jwt_conf = jwt::new(jwt_secret);

        let original_jwt = jwt_conf.new(
            jwt::JwtType::RefreshToken,
            "admin",
        );

        // Both ways viable
        let serialized_jwt = jwt_conf.serilize(original_jwt.clone());
        let serialized_jwt2 = original_jwt.clone().serialize(&jwt_conf);
        assert_eq!(serialized_jwt, serialized_jwt2);



        let deserialized_jwt = match serialized_jwt.deserialize(&jwt_conf) {
            Ok(val) => val,
            Err(err) => panic!("Deserialization error: {}", err),
        };
        assert_eq!(original_jwt, deserialized_jwt);
    }

    #[test]
    fn converting_to_string_and_back() {
        let jwt_secret = "Secret string";
        let jwt_conf = jwt::new(jwt_secret);
        let original_jwt = jwt_conf.new(
            jwt::JwtType::RefreshToken,
            "admin",
        );

        let serialized_jwt = original_jwt.serialize(&jwt_conf);
        let jwt_string = serialized_jwt.to_string();

        // All ways viable
        let re_read_jwt: jwt::JwtSerialized = jwt_string.clone().into();
        let re_read_jwt2 = jwt::JwtSerialized::from(jwt_string.clone());
        let re_read_jwt3 = jwt_conf.from_str(jwt_string);
        assert_eq!(re_read_jwt, re_read_jwt2);
        assert_eq!(re_read_jwt2, re_read_jwt3);

        let invalid_jwt = jwt_conf.from_str("This isn't a jwt".to_string());
        assert!(jwt_conf.deserialize(invalid_jwt).is_err());
    }

    /// Here i have 2 jwt secrets and 2 configs, one config will serialize using one key and deserialize with the other and the second config will do the vice versa. It doesn't matter with which config created the jwt
    #[test]
    fn changing_secrets() {
        let jwt_secret_a = "The first jwt secret";
        let jwt_secret_b = "The second jwt secret";

        let jwt_conf_a_to_b = jwt::new(jwt_secret_a)
            .decoding_secret(jwt_secret_b);

        let jwt_conf_b_to_a = jwt_conf_a_to_b
            .clone()
            .encoding_secret(jwt_secret_b)
            .decoding_secret(jwt_secret_a);

        let original_jwt = jwt_conf_a_to_b.new(
            jwt::JwtType::RefreshToken,
            "admin",
        );

        let serialized_with_a = jwt_conf_a_to_b.serilize(original_jwt.clone());
        let serialized_with_b = jwt_conf_b_to_a.serilize(original_jwt.clone());
        assert_ne!(serialized_with_a, serialized_with_b);

        // Deserialize with a
        let deserialized_with_a = jwt_conf_a_to_b.deserialize(serialized_with_b);
        assert!(deserialized_with_a.is_ok());
        let deserialized_with_a = deserialized_with_a.unwrap();
        assert_eq!(original_jwt, deserialized_with_a);

        // Deserialize with b
        let deserialized_with_b = jwt_conf_b_to_a.deserialize(serialized_with_a);
        assert!(deserialized_with_b.is_ok());
        let deserialized_with_b = deserialized_with_b.unwrap();
        assert_eq!(original_jwt, deserialized_with_b);

        assert_eq!(deserialized_with_a, deserialized_with_b);
    }
}