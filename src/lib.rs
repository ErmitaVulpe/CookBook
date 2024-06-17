#[cfg(feature="ssr")]
use {
    diesel::prelude::*,
    diesel::r2d2,
    std::fs,
    crate::{cdn::Cdn, auth::JwtConfig},
};

pub mod app;
pub mod api;
pub mod pages;
pub mod md_parser;

#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod schema;
#[cfg(feature = "ssr")]
pub mod cdn;

pub const PUBLIC_URL: &str = env!("BUILD_PUBLIC_URL");

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    use leptos::*;

    console_error_panic_hook::set_once();

    mount_to_body(App);
}

#[cfg(feature = "ssr")]
pub struct AppData {
    pool: r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    pub admin: api::auth::User,
    pub cdn: cdn::Cdn,
    pub jwt: JwtConfig,
}

#[cfg(feature="ssr")]
impl AppData {
    pub fn new() -> Self {
        use std::{env, process::exit};
        
        let pool = {
            let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| {
                eprintln!("DATABASE_URL var not set");
                exit(1);
            });

            if fs::metadata(&db_path).is_err() {
                eprintln!("Database file not found");
                exit(1);
            }

            r2d2::Pool::builder().build(r2d2::ConnectionManager::<SqliteConnection>::new(db_path))
            .unwrap_or_else(|e| {
                eprintln!("Error opening database:\n{e}");
                exit(1);
            })
        };

        let admin = api::auth::UserRaw::new(
            &env::var("ADMIN_USERNAME").unwrap_or_else(|_| {
                eprintln!("ADMIN_USERNAME var not set");
                exit(1);
            }),
            &env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
                eprintln!("ADMIN_PASSWORD var not set");
                exit(1);
            }),
        ).hash();

        let cdn = Cdn::new(&env::var("CDN_PATH").unwrap_or_else(|_| {
            println!("CDN_PATH var not set. Defaulting to cdn/");
            "cdn/".to_owned()
        }));
        
        let jwt = JwtConfig::default();

        AppData {
            pool,
            admin,
            cdn,
            jwt,
        }
    }

    pub fn get_conn(&self) -> Result<
        r2d2::PooledConnection<r2d2::ConnectionManager<diesel::SqliteConnection>>,
        leptos::ServerFnError
    > {
        self.pool.get().map_err(|e| {
            leptos::ServerFnError::new(e)
        })
    }
}

#[cfg(feature="ssr")]
impl Default for AppData {
    fn default() -> Self {
        Self::new()
    }
}
