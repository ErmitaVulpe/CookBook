use actix_web::{cookie, HttpResponse, HttpRequest, web};
use serde::Deserialize;

use super::db::prelude::*;
use super::{auth, db, models};
use super::auth::jwt::JwtType;
use super::auth::CookieName;


pub fn auth(cfg: &mut web::ServiceConfig) {
    cfg
        .service(log_in)
        .service(log_out)
        .service(refresh)
        .service(change_password);
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
        let jwt = app_data.jwt_conf.jwt_from_str(val.value().to_string());
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

    let jwt_data = jwt_conf.new_jwt(
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
    let jwt_conf = &app_data.jwt_conf;

    // Invalidate the user refresh token
    let refresh_token = match req.cookie(&CookieName::RefreshToken.to_string()) {
        Some(val) => val.value().to_string(),
        None => return HttpResponse::Unauthorized().finish(),
    };
    let jwt = jwt_conf.jwt_from_str(refresh_token);
    jwt_conf.invalidate(jwt);

    // Invalidate the user access token if exists
    if let Some(val) = req.cookie(&CookieName::AccessToken.to_string()) {
        let jwt_string = val.value();
        let jwt = jwt_conf.jwt_from_str(
            jwt_string.to_string());
        jwt_conf.invalidate(jwt);
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

    let jwt_conf = &app_data.jwt_conf;

    // Parse refresh token
    let refresh_jwt = jwt_conf.jwt_from_str(refresh_token);
    let claims = match jwt_conf.validate(refresh_jwt) {
        Some(val) => val,
        None => return HttpResponse::Unauthorized().finish(),
    };

    // Invalidate old refresh token if exists
    if let Some(val) = req.cookie(&CookieName::AccessToken.to_string()) {
        let jwt = jwt_conf.jwt_from_str(val.value().to_string());
        jwt_conf.invalidate(jwt);
    }

    // Create an register the access token
    let access_jwt = jwt_conf.new_jwt(
        JwtType::AccessToken,
        &claims.get_username()
    );
    let expiration_time = access_jwt.get_expiration();
    let serialized_access_jwt = jwt_conf.register(access_jwt).to_string();

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


#[derive(Deserialize)]
struct ChangePsswordData {
    password: String,
    new_password: String,
}

#[actix_web::post("/change_password")]
async fn change_password(
    req: HttpRequest,
    app_data: web::Data<models::AppData>,
    change_password_data: web::Json<ChangePsswordData>,
) -> HttpResponse {
    let jwt_conf = &app_data.jwt_conf;

    // Try to get the refresh token
    let refresh_token = match req.cookie(&CookieName::RefreshToken.to_string()) {
        Some(val) => val.value().to_string(),
        None => return HttpResponse::Unauthorized().finish(),
    };
    // Validate and deserialize it
    let jwt = jwt_conf.jwt_from_str(refresh_token);
    let claims = match jwt_conf.validate(jwt) {
        Some(val) => val,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let mut conn: db::Conn = super::get_conn!(app_data.pool);

    // Query the db
    let query_result = users_dsl::users
        .select(users_dsl::password_hash)
        .find(claims.get_username())
        .first(&mut conn);

    // If the logged in user not found, retune an internal error
    let pw_hash: String = match query_result {
        Ok(val) => val,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Check if the passed password is valid
    if ! auth::verify_password(&change_password_data.password, &pw_hash) {
        return HttpResponse::Unauthorized().finish();
    }

    // Hash and set the new password
    let new_pw_hash = auth::hash_password(&change_password_data.new_password);
    let query_result = diesel::update(users_dsl::users.find(claims.get_username()))
        .set(users_dsl::password_hash.eq(new_pw_hash))
        .execute(&mut conn);

    // Check for an error
    if query_result.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}
