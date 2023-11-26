#[allow(unused_imports)]
use crate::{auth, db, macros, models, schema, unwrap_pretty, validating};
use actix_web::web;

// Macros to use inside of this module
mod macro_mod {
    macro_rules! get_conn {
        ($pool:expr) => {
            match $pool.get() {
                Ok(val) => val,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }
        };
    }
    pub(crate) use get_conn;
}
use macro_mod::*;




mod auth_endpoint;

pub fn api_v1(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::scope("/auth").configure(auth_endpoint::auth));



        // .service(auth_endpoint::auth);

        // .route("auth", web::post().to(endpoints::auth_handler))
        // .route("auth/", web::post().to(endpoints::auth_handler))

        // .route("users", web::post().to(endpoints::users_handler))
        // .route("users/", web::post().to(endpoints::users_handler))

        // .route("cards", web::post().to(endpoints::cards_handler))
        // .route("cards/", web::post().to(endpoints::cards_handler));
}