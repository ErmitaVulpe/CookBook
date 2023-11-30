use actix_web::{cookie, App, HttpResponse, HttpRequest, web};
use serde::Serialize;

use super::db::prelude::*;
use super::{auth, db, models};
use super::auth::CookieName;


pub fn me(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_me);
}


#[actix_web::get("")]
async fn get_me(
    req: HttpRequest,
    app_data: web::Data<models::AppData>,
) -> HttpResponse {
    // Data that will be returned if successful
    #[derive(Serialize)]
    struct ResponseData {
        username: String,
    }

    let jwt_conf = &app_data.jwt_conf;
    let claims = super::check_access_token!(jwt_conf, req);

    let response_data = ResponseData {
        username: claims.get_username(),
    };

    HttpResponse::Ok().json(response_data)
}
