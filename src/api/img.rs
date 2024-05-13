#[cfg(feature="ssr")]
use actix_web::{get, post, HttpResponse, Responder, web};

// #[cfg(feature="ssr")]
// pub type cdn_meta = std::collections::HashMap<(u32, u32), FileExtensions>;

#[cfg(feature="ssr")]
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_img)
        .service(upload_img)
    ;
}

#[cfg(feature="ssr")]
#[get("/{folder}/{img}")]
async fn get_img(
    _app_data: web::Data<crate::AppData>,
    _path: web::Path<(u32, u32)>,
) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[cfg(feature="ssr")]
#[post("/upload_img")]
async fn upload_img(
    _app_data: web::Data<crate::AppData>,
    _path: web::Path<(u32, u32)>,
) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
