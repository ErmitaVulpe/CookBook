use serde::{Deserialize, Serialize};
use serde_repr::*;
use leptos::*;
use leptos::server_fn::codec;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserRaw {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "p")]
    pub password: String,
}

impl UserRaw {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }

    #[cfg(feature = "ssr")]
    pub fn hash(self) -> User {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.password.as_bytes());
        let pw_hash = hasher.finish();

        User {
            username: self.username,
            pw_hash,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Deserialize_repr, Serialize_repr, PartialEq)]
pub enum LoggedStatus {
    LoggedIn,
    LoggedOut,
}

#[cfg(feature = "ssr")]
pub use ssr::*;
#[cfg(feature = "ssr")]
mod ssr {
    pub use std::hash::{DefaultHasher, Hasher};

    #[derive(Clone, Debug)]
    pub struct User {
        pub username: String,
        pub pw_hash: u64,
    }

    impl User {
        pub fn new(username: &str, password: &str) -> Self {
            let mut hasher = DefaultHasher::new();
            hasher.write(password.as_bytes());
            let pw_hash = hasher.finish();

            Self {
                username: username.to_owned(),
                pw_hash,
            }
        }
    }

    impl PartialEq for User {
        fn eq(&self, other: &Self) -> bool {
            let mut hasher1 = DefaultHasher::new();
            let mut hasher2 = hasher1.clone();

            hasher1.write(self.username.as_bytes());
            let h1 = hasher1.finish();

            hasher2.write(other.username.as_bytes());
            let h2 = hasher2.finish();

            h1 == h2 && self.pw_hash == other.pw_hash
        }
    }
}

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn log_in(user: UserRaw) -> Result<LoggedStatus, ServerFnError> {
    use actix_web::{cookie, cookie::Cookie, http::header, http::{header::HeaderValue, StatusCode}};
    use leptos_actix::{extract, ResponseOptions};
    use crate::auth::{Claims, Permissions};

    let app_data = extract::<actix_web::web::Data<crate::AppData>>().await
        .map(|i| i.into_inner())?;
    let response = expect_context::<ResponseOptions>();
    let user = user.hash();

    Ok(match user == app_data.admin {
        false => LoggedStatus::LoggedOut,
        true => {
            let token = app_data.jwt.generate(Claims::new(Permissions::Admin))
                .map_err(|_| ServerFnError::new("Error generating a token".to_string()))?;

            let cookie = Cookie::build("jwt", &token)
                .max_age(cookie::time::Duration::hours(6))
                .http_only(true)
                .same_site(cookie::SameSite::Strict)
                .path("/")
                .finish();

            let cookie_val = HeaderValue::from_str(&cookie.to_string())
                .map_err(|_| ServerFnError::new("Error generating a token".to_string()))?;

            response.insert_header(header::SET_COOKIE, cookie_val);
            response.set_status(StatusCode::OK);
            LoggedStatus::LoggedIn
        },
    })
}

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn log_out() -> Result<(), ServerFnError> {
    use actix_web::{cookie::Cookie, http::header, http::header::HeaderValue};
    use leptos_actix::ResponseOptions;
    use leptos::server_fn::error::NoCustomError;

    let response = expect_context::<ResponseOptions>();
    
    let mut cookie = Cookie::named("jwt");
    cookie.make_removal();

    let cookie_val = HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ServerFnError::<NoCustomError>::ServerError("Error generating a token".to_string()))?;
    response.insert_header(header::SET_COOKIE, cookie_val);

    Ok(())
}

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn is_logged() -> Result<LoggedStatus, ServerFnError> {
    use actix_web::HttpRequest;

    let app_data = super::extract_app_data().await?;
    let request = expect_context::<HttpRequest>();
    Ok(check_if_logged(&app_data.jwt, &request))
}

#[cfg(feature = "ssr")]
pub fn check_if_logged(jwt: &crate::auth::JwtConfig, req: &actix_web::HttpRequest) -> LoggedStatus {
    let cookie = match req.cookie("jwt") {
        Some(val) => val,
        None => return LoggedStatus::LoggedOut,
    };
    let token = cookie.value();

    match jwt.decode(token) {
        Ok(val) => match val.permissions == crate::auth::Permissions::Admin {
            true => LoggedStatus::LoggedIn,
            false => LoggedStatus::LoggedOut,
        },
        Err(_) => LoggedStatus::LoggedOut,
    }
}
