mod api;
mod auth;
mod db;
mod schema;
mod setup;
mod macros;
mod models;
mod unwrap_pretty;
mod validating;

use dotenv::dotenv;
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, RwLock};
use std::env;
use macros::exit_with_error;
use unwrap_pretty::UnwrapPretty;


#[actix_web::get("/")]
pub async fn hello() -> impl actix_web::Responder {
    "Hello, World!"
}


#[actix_web::main]
async fn main() {
    dotenv().ok();
    
    let mut database_path = env::var("CB_DATABASE_PATH").unwrap_or("database.db".to_owned());
    let mut socket = match env::var("CB_SOCKET") {
        Ok(val) => Some(val),
        Err(_) => None,
    };
    let mut jwt_secret = None;

    // Parse arguments
    let mut iter = env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-d" | "--database" => {
                match iter.next() {
                    Some(path) => database_path = path,
                    None => exit_with_error!("No new database path specified"),
                };
            }
            "-s" | "--setup" => setup::setup(&database_path),
            "-s:ndb" => {
                let database_path = match iter.next() {
                    Some(path) => path,
                    None => exit_with_error!("No new database path specified"),
                };

                let admin_pw = match iter.next() {
                    Some(pw) => pw,
                    None => exit_with_error!("No admin password specified"),
                };

                setup::new_db_file(&database_path, &admin_pw);
            }
            "-s:nu" => {
                let username = match iter.next() {
                    Some(pw) => pw,
                    None => exit_with_error!("No username for the new account specified"),
                };

                let pw = match iter.next() {
                    Some(pw) => pw,
                    None => exit_with_error!("No password specified"),
                };

                setup::new_user(&database_path, &username, &pw);
            }
            "-s:ni" => {
                let name = match iter.next() {
                    Some(pw) => pw,
                    None => exit_with_error!("No ingredient name specified"),
                };

                setup::new_ingredient(&database_path, &name);
            }
            "-s:ri" => {
                let name = match iter.next() {
                    Some(pw) => pw,
                    None => exit_with_error!("No ingredient name specified"),
                };

                setup::remove_ingredient(&database_path, &name);
            }
            "-s:S" => {
                let sock = match iter.next() {
                    Some(value) => value,
                    None => exit_with_error!("No socket specified"),
                };

                setup::set_socket(&database_path, &sock);
            }
            "-s:j" => {
                let jwt_secret = match iter.next() {
                    Some(value) => value,
                    None => exit_with_error!("No jwt secret specified"),
                };

                setup::new_jwt_secret(&database_path, Some(jwt_secret));
            }
            "-s:j:rand" => {
                setup::new_jwt_secret(&database_path, None);
            }
            "-S" | "--socket" => {
                socket = match iter.next() {
                    Some(value) => Some(value),
                    None => exit_with_error!("No socket specified"),
                };
            }
            "-j" | "--jwt" => {
                jwt_secret = match iter.next() {
                    Some(value) => Some(value),
                    None => exit_with_error!("No jwt secret specified"),
                };
            }
            "-e" | "--exit" => {
                std::process::exit(0);
            }
            "-v" | "--version" => {
                println!("CookBook by FullStackBros v{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => exit_with_error!("Parameter not found: {}", arg)
        }
    }


    let pool: db::Pool = setup::validate_db(&database_path);
    let mut conn: db::Conn = pool.get().unwrap();
    
    let socket = match socket {
        Some(value) => {
            if validating::is_valid_socket(&value) {
                value
            } else {
                exit_with_error!("Invalid socket");
            }
        },
        None => {
            db::key_value::get(&mut conn, "socket").unwrap_pretty(
                "Setting for socket not found. Try setting it using the \"-s:S\" flag or just use \"-S\" for a temporary socket")
        }
    };


    let jwt_conf = {
        let jwt_secret = match jwt_secret {
            Some(value) => value,
            None => {
                db::key_value::get(&mut conn, "jwt_secret").unwrap_pretty(
                    "Setting for jwt secret not found. Try setting it in the setup menu (-s flag) or just use \"-j\" for a temporary jwt secret")
            }
        };

        auth::jwt::new(&jwt_secret)
    };


    let app_data = models::AppData {
        pool,
        jwt_conf,
    };

    // Set up web server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .service(hello)
            .service(web::scope("/api/v1").configure(api::api_v1))
    });

    // Bind web server to a socket
    let bound_server = {
        #[cfg(not(feature = "ssl"))]
        let try_bind = server.bind(socket.clone());

        // Set up the ssl config
        #[cfg(feature = "ssl")]
        let builder = {
            use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            // Get key path
            let key_path = db::key_value::get(&mut conn, "ssl_key_path")
                .unwrap_pretty("Couldn't get the ssl key path");
            // Check if file exists
            if std::fs::metadata(key_path.clone()).is_err() {
                exit_with_error!("Ssl key file not found at \"{}\"", key_path);
            }
            // Set ssl key
            if builder.set_private_key_file(key_path, SslFiletype::PEM).is_err() {
                exit_with_error!("Ssl key file invalid");
            }

            // Get cert path
            let cert_path = db::key_value::get(&mut conn, "ssl_cert_path")
                .unwrap_pretty("Couldn't get the ssl cert path");
            // Check if file exists
            if std::fs::metadata(cert_path.clone()).is_err() {
                exit_with_error!("Ssl cert file not found at \"{}\"", cert_path);
            }
            // Set ssl cert
            if builder.set_certificate_chain_file(cert_path).is_err() {
                exit_with_error!("Ssl cert file invalid");
            }

            builder
        };

        #[cfg(feature = "ssl")]
        let try_bind = server.bind_openssl(socket.clone(), builder);

        try_bind.unwrap_pretty("Couldn't bind on the specified socket")
    };

    // Start the web server
    println!("Strating server at: {}", socket);

    bound_server.run()
    .await
    .unwrap_or_else(|err| exit_with_error!("Encountered an unexpected error: {}", err));

    println!("Server stopped");
}
