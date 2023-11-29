#[allow(unused_imports)]
use actix_web::{cookie, App, HttpServer, HttpResponse, HttpRequest, web};
use serde::Deserialize;

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
    username: String,
    password: String,
}

#[actix_web::post("/log_in")]
async fn log_in(
    req: HttpRequest,
    app_data: web::Data<models::AppData>,
    credentials: web::Json<Credentials>,
) -> HttpResponse {
    // Check if user is already logged in
    if let Some(val) = req.cookie(&CookieName::RefreshToken.to_string()) {
        let jwt = app_data.jwt_conf.from_str(val.value().to_string());
        if app_data.jwt_conf.validate(jwt).is_none() {
            return HttpResponse::Ok().body("Already logged in");
        }
    }

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
    let jwt_string = jwt_conf.register(jwt_data).to_string();

    let cookie = cookie::Cookie::build(CookieName::RefreshToken.to_string(), jwt_string)
        .path("/auth")
        .expires(chrono_to_cookie_time!(expiration_time))
        .secure(true)
        .http_only(true) 
        .finish();

    HttpResponse::Ok().cookie(cookie).finish()
}


#[actix_web::get("/log_out")]
async fn log_out(
    req: HttpRequest,
    app_data: web::Data<models::AppData>,
) -> HttpResponse {
    // Invalidate the refresh token
    let refresh_token = match req.cookie(&CookieName::RefreshToken.to_string()) {
        Some(val) => val.value().to_string(),
        None => return HttpResponse::Unauthorized().finish(),
    };
    let jwt = app_data.jwt_conf.from_str(refresh_token);
    app_data.jwt_conf.invalidate(jwt);

    // Invalidate the access token if exists
    if let Some(val) = req.cookie(&CookieName::AccessToken.to_string()) {
        let jwt_string = val.value();
        let jwt = app_data.jwt_conf.from_str(
            jwt_string.to_string());
        app_data.jwt_conf.invalidate(jwt);
    }
    
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
async fn refresh(
    query_params: web::Query<RedirectParams>,
    req: HttpRequest,
    app_data: web::Data<models::AppData>,
) -> HttpResponse {
    // Try to get the refresh token
    let refresh_token = match req.cookie(&CookieName::RefreshToken.to_string()) {
        Some(val) => val.value().to_string(),
        None => return HttpResponse::Unauthorized().finish(),
    };

    // Parse refresh token
    let refresh_jwt = app_data.jwt_conf.from_str(refresh_token);
    let claims = match app_data.jwt_conf.validate(refresh_jwt) {
        Some(val) => val,
        None => return HttpResponse::Unauthorized().finish(),
    };

    // Invalidate old refresh token if exists
    if let Some(val) = req.cookie(&CookieName::AccessToken.to_string()) {
        let jwt = app_data.jwt_conf.from_str(val.value().to_string());
        app_data.jwt_conf.invalidate(jwt);
    }

    // Create an register the access token
    let access_jwt = app_data.jwt_conf.new(
        JwtType::AccessToken,
        &claims.get_username()
    );
    let expiration_time = access_jwt.get_expiration();
    let serialized_access_jwt = app_data.jwt_conf.register(access_jwt).to_string();

    // Build a cookie
    let access_cookie = cookie::Cookie::build(
        CookieName::AccessToken.to_string(),
        serialized_access_jwt
    )
        .path("/")
        .expires(chrono_to_cookie_time!(expiration_time))
        .secure(true)
        .http_only(true) 
        .finish();

    // Send a response
    if let Some(redirect_path) = &query_params.from {
        HttpResponse::Found()
            .cookie(access_cookie)
            .append_header(("Location", redirect_path.as_str()))
            .finish()
    } else {
        HttpResponse::Ok().cookie(access_cookie).finish()
    }
}