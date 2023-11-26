// use diesel::prelude::*;
use crate::exit_with_error;
use diesel::{r2d2::{self, ConnectionManager}, SqliteConnection};
use prelude::*;

pub const SQL_DECLARATION: &str = {
    use constcat::concat;

    const NO_SSL_KEY_VALUES: &str = r#"("socket", "0.0.0.0:80")"#;
    const SSL_KEY_VALUES: &str = r#"("ssl_cert_path", "./cert.pem"),
("ssl_key_path", "./key.pem"),
("socket", "0.0.0.0:443")"#;
    
    concat!(
        include_str!("../migrations/2023-11-14-233125_init/up.sql"), // Raw sql
        "INSERT INTO key_value (key, value) VALUES ", // Template just to not repeat myself
        if cfg!(feature = "ssl") { SSL_KEY_VALUES } else { "" },
        if cfg!(not(feature = "ssl")) { NO_SSL_KEY_VALUES } else { "" },
        ";", // Trailing semicolon
    )
};

/// ## Alias for connection pool type
pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type Conn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>;

/// ## Function for establishing a pool connection to the database
///
/// ### Returns
/// **db::Pool** type, aka **diesel::r2d2::Pool\<diesel::r2d2::ConnectionManager\<diesel::prelude::SqliteConnection\>\>**
pub fn establish_connection(url: String) -> Pool {
    r2d2::Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(url))
        .unwrap_or_else(|err| exit_with_error!("Couldn't create a db connection pool.:\n{}", err))
}


// my onw prelude
pub mod prelude {
    use crate::schema;

    pub use diesel::prelude::*;

    pub use schema::ammounts::dsl as ammounts_dsl;
    pub use schema::ingredients::dsl as ingredients_dsl;
    pub use schema::key_value::dsl as key_value_dsl;
    pub use schema::recipes::dsl as recipes_dsl;
    pub use schema::users::dsl as users_dsl;
}


pub mod key_value {
    use crate::{db::Conn, models};
    use super::key_value_dsl;
    use diesel::prelude::*;

    pub fn get(conn: &mut Conn, key: &str) -> Result<String, diesel::result::Error> {
        let result: models::KeyValue = key_value_dsl::key_value
            .find(key)
            .first::<models::KeyValue>(conn)?;
        Ok(result.value)
    }

    pub fn set(conn: &mut Conn, key: &str, value: &str) -> Result<(), diesel::result::Error> {
        diesel::replace_into(key_value_dsl::key_value)
            .values(&models::KeyValue {
                key: key.to_owned(),
                value: value.to_owned(),
            })
            .execute(conn)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove(conn: &mut Conn, key: &str) -> Result<(), diesel::result::Error> {
        diesel::delete(key_value_dsl::key_value.filter(key_value_dsl::key.eq(key)))
            .execute(conn)?;
        Ok(())
    }
}

