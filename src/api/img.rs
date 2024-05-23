#[cfg(feature="ssr")]
use actix_web::{Error as ActixError, get, put, HttpResponse, Responder, web};
#[cfg(feature="ssr")]
use actix_multipart::Multipart;
#[cfg(feature="ssr")]
use futures_util::TryStreamExt as _;
use leptos::ServerFnError;
use server_fn::error::NoCustomError;
use web_sys::{FormData, File};

#[cfg(feature="ssr")]
use super::auth::{check_if_logged, LoggedStatus};

#[cfg(feature="ssr")]
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_img)
        .service(upload_icon_private)
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
#[put("/upload_icon")]
async fn upload_icon_private(
    app_data: web::Data<crate::AppData>,
    req: actix_web::HttpRequest,
    mut payload: Multipart,
)  -> Result<HttpResponse, ActixError> {
    if check_if_logged(&app_data.jwt, &req) != LoggedStatus::LoggedIn {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let mut recipe_name = String::new();
    let mut raw_icon_bytes = Vec::new();

    while let Some(mut field) = payload.try_next().await? {
        match field.name() {
            "r" => while let Some(chunk) = field.try_next().await? {
                let string = match String::from_utf8((*chunk).to_owned()) {
                    Ok(val) => val,
                    Err(_) => return Ok(HttpResponse::BadRequest().finish()),
                };
                recipe_name.push_str(&string);
                if recipe_name.len() > super::recipes::MAX_RECIPE_NAME_LENGHT {
                    return Ok(HttpResponse::PayloadTooLarge().finish())
                }
            },
            "d" => while let Some(chunk) = field.try_next().await? {
                raw_icon_bytes.extend_from_slice(&*chunk);
                if recipe_name.len() > crate::cdn::MAX_IMAGE_SIZE {
                    return Ok(HttpResponse::PayloadTooLarge().finish())
                }
            },
            _ => return Ok(HttpResponse::BadRequest().finish())
        }
    }

    println!("name: {recipe_name}");
    println!("len: {}", raw_icon_bytes.len());

    // TODO convert and save the icon

    Ok(HttpResponse::Ok().finish())
}

pub async fn upload_icon(host: &str, recipe_name: &str, icon: &File) -> Result<Result<(), super::Error>, ServerFnError> {
    let url = format!(
        "{host}/cdn/img/upload_icon");

    let form_data = FormData::new().unwrap();
    form_data.append_with_str("r", recipe_name).unwrap();
    form_data.append_with_blob("d", icon).unwrap();
    let resp = reqwasm::http::Request::put(&url)
        .body(form_data)
        .send()
        .await;

    match resp {
        Err(err) => Err(ServerFnError::Request(err.to_string())),
        Ok(resp) => {
            match resp.status() {
                200..=299 => Ok(Ok(())),
                400 => Err(ServerFnError::<NoCustomError>::Args(resp.status_text())),
                403 => Ok(Err(super::Error::Unauthorized)),
                413 => Err(ServerFnError::<NoCustomError>::Args(resp.status_text())),
                _ => Err(ServerFnError::<NoCustomError>::WrappedServerError(NoCustomError)),
            }
        },
    }
}


#[cfg(feature="ssr")]
#[put("/upload_img")]
async fn upload_img(
    _app_data: web::Data<crate::AppData>,
    _path: web::Path<(u32, u32)>,
) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
