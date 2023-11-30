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
    
    macro_rules! check_access_token {
        ($jwt_conf:expr, $req:expr) => {{
            // Try to get the access token
            let access_token = match $req.cookie(&CookieName::AccessToken.to_string()) {
                Some(val) => val.value().to_string(),
                None => return HttpResponse::Unauthorized().finish(),
            };
            // Validate and deserialize it
            let jwt = $jwt_conf.jwt_from_str(access_token);
            match $jwt_conf.validate(jwt) {
                Some(val) => val,
                None => return HttpResponse::Unauthorized().finish(),
            }
        }};
    }
    pub(crate) use check_access_token;
}
use macro_mod::*;




mod auth_endpoint;
mod me_endpoint;

pub fn api_v1(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::scope("/auth").configure(auth_endpoint::auth))
        .service(web::scope("/me").configure(me_endpoint::me));



        // .service(auth_endpoint::auth);

        // .route("auth", web::post().to(endpoints::auth_handler))
        // .route("auth/", web::post().to(endpoints::auth_handler))

        // .route("users", web::post().to(endpoints::users_handler))
        // .route("users/", web::post().to(endpoints::users_handler))

        // .route("cards", web::post().to(endpoints::cards_handler))
        // .route("cards/", web::post().to(endpoints::cards_handler));
}