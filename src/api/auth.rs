use serde::{Deserialize, Serialize};
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

            hasher2.write(self.username.as_bytes());
            let h2 = hasher1.finish();

            h1 == h2 && self.pw_hash == other.pw_hash
        }
    }
}

#[server(input = codec::Cbor)]
pub async fn log_in(_user: UserRaw) -> Result<(), ServerFnError> {
    // use actix_web::{cookie::Cookie, http::header, http::header::HeaderValue};
    // use actix_web::web::Data;
    // use leptos_actix::extract;
    // use leptos_actix::ResponseOptions;
    // use actix_web::dev::ConnectionInfo;
    // use actix_web::http::StatusCode;

    // // pull ResponseOptions from context
    // let response = expect_context::<ResponseOptions>();

    // let connection: ConnectionInfo = extract().await?;
    // println!("connection = {connection:?}");

    // // set the HTTP status code
    // response.set_status(StatusCode::IM_A_TEAPOT);

    // // set a cookie in the HTTP response
    // let mut cookie = Cookie::build("biscuits", "yes").finish();
    // if let Ok(cookie) = HeaderValue::from_str(&cookie.to_string()) {
    //     response.insert_header(header::SET_COOKIE, cookie);
    // };

    Ok(())
}

