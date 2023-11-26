#[allow(unused_imports)]
use actix_web::{cookie, App, HttpServer, HttpResponse, web};
use serde::Deserialize;
use chrono::{DateTime, Utc};

use super::db::prelude::*;
use super::{auth, db, models};
use super::auth::jwt::JwtType;
use super::auth::CookieName;


pub fn auth(cfg: &mut web::ServiceConfig) {
    cfg
        .service(log_in)
        .service(log_out)
        .service(refresh);
}


macro_rules! chrono_to_cookie_time {
    ($chrono_time:expr) => {
        cookie::time::OffsetDateTime::from_unix_timestamp($chrono_time.timestamp()).unwrap()
    };
}

#[derive(Deserialize)]
struct Credentials {
    // Your data structure here
    username: String,
    password: String,
}

#[actix_web::post("/log_in")]
async fn log_in(
    app_data: web::Data<models::AppData>,
    credentials: web::Json<Credentials>,
) -> HttpResponse {
    let mut conn: db::Conn = super::get_conn!(app_data.pool);

    // Check if specified user exists
    let query_result = users_dsl::users
        .find(&credentials.username)
        .first::<models::User>(&mut conn);

    // Get user data from db
    let user_data: models::User = match query_result {
        Ok(val) => val,
        Err(diesel::result::Error::NotFound) => return HttpResponse::Unauthorized().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Verify password
    if ! auth::verify_password(&credentials.password, &user_data.password_hash) {
        return HttpResponse::Unauthorized().finish();
    }

    // Auth successful, respond with a cookie
    let jwt_conf = &app_data.jwt_conf;

    let jwt_data = jwt_conf.new(
        JwtType::RefreshToken,
        &credentials.username
    );
    let expiration_time = jwt_data.get_expiration();
    let jwt_string = jwt_conf.serilize(jwt_data).to_string();

    let cookie = cookie::Cookie::build(CookieName::RefreshToken.to_string(), jwt_string)
        .path("/auth")
        .expires(chrono_to_cookie_time!(expiration_time))
        .secure(true)
        .http_only(true) 
        .finish();

    HttpResponse::Ok().cookie(cookie).finish() // TODO redirect to refresh
}

#[actix_web::get("/log_out")]
async fn log_out(
    // app_data: web::Data<models::AppData>,
) -> HttpResponse {
    // TODO Implement token invalidation here
    
    let refresh_cookie = cookie::Cookie::build(CookieName::RefreshToken.to_string(), "")
        .path("/auth")
        .expires(cookie::time::OffsetDateTime::UNIX_EPOCH)
        .secure(true)
        .http_only(true) 
        .finish();

    let access_cookie = cookie::Cookie::build(CookieName::AccessToken.to_string(), "")
        .path("/")
        .expires(cookie::time::OffsetDateTime::UNIX_EPOCH)
        .secure(true)
        .http_only(true) 
        .finish();

    HttpResponse::Ok().cookie(refresh_cookie).cookie(access_cookie).finish()
}

// TODO implement refresh endpoint


#[derive(Debug, Deserialize)]
struct RedirectParams {
    from: Option<String>,
}

#[actix_web::get("/refresh")]
async fn refresh(query_params: web::Query<RedirectParams>) -> HttpResponse {
    if let Some(user_name) = &query_params.from {
        HttpResponse::Found()
            .append_header(("Location", user_name.as_str()))
            .finish()
    } else {
        HttpResponse::Ok().body("Hello, anonymous!")
    }
}