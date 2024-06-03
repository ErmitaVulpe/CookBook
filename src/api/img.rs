#[cfg(feature="ssr")]
use actix_web::{
    Error as ActixError,
    HttpResponse,
    get,
    put,
    Responder,
    web
};
#[cfg(feature="ssr")]
use actix_multipart::Multipart;
#[cfg(feature="ssr")]
use futures_util::TryStreamExt as _;
use leptos::{server, ServerFnError};
use server_fn::{codec, error::NoCustomError};
use web_sys::{FormData, File, window};

#[cfg(feature="ssr")]
use crate::cdn::CdnError;

#[cfg(feature="ssr")]
use super::{
    auth::{check_if_logged, LoggedStatus},
    extract_app_data,
};
use super::Error;

#[cfg(feature="ssr")]
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_img)
        .service(upload_icon_private)
        .service(upload_images_private)
    ;
}


#[cfg(feature="ssr")]
#[get("/get/{recipe}/{image}")]
async fn get_img(
    app_data: web::Data<crate::AppData>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (recipe, image) = path.into_inner();

    if let Some('.') = recipe.chars().next() {
        return HttpResponse::NotFound().finish();
    }

    if let Some('.') = image.chars().next() {
        return HttpResponse::NotFound().finish();
    }

    let path = format!(
        "{}{}/{}",
        app_data.cdn.path,
        recipe,
        image,
    );
    match std::fs::read(path) {
        Err(_) => HttpResponse::NotFound().finish(),
        Ok(image_bytes) => {
            let hash = xxhash_rust::xxh3::xxh3_64(&image_bytes);

            HttpResponse::Ok()
                .content_type("image/webp")
                .append_header(("Etag", format!("\"{hash:x}\"")))
                .append_header(("Access-Control-Allow-Origin", "*"))
                .append_header(("Cross-Origin-Resource-Policy", "cross-origin"))
                .body(image_bytes)
        }
    }
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
                raw_icon_bytes.extend_from_slice(&chunk);
                if recipe_name.len() > crate::cdn::MAX_IMAGE_SIZE {
                    return Ok(HttpResponse::PayloadTooLarge().finish())
                }
            },
            _ => return Ok(HttpResponse::BadRequest().finish())
        }
    }

    let result = app_data.cdn.transaction(|cdn| {
        cdn.upload_icon(&recipe_name, &raw_icon_bytes)
    });

    match result {
        Ok(()) => Ok(HttpResponse::Ok().finish()),
        Err(e) => Ok(e.into()),
    }
}

pub async fn upload_icon(recipe_name: &str, icon: &File) -> Result<Result<(), super::Error>, ServerFnError> {
    let location = window().unwrap().location();
    let url = format!(
        "{}//{}/cdn/img/upload_icon",
        location.protocol().unwrap(),
        location.host().unwrap(),
    );

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
#[put("/upload_images")]
async fn upload_images_private(
    app_data: web::Data<crate::AppData>,
    req: actix_web::HttpRequest,
    mut payload: Multipart,
)  -> Result<HttpResponse, ActixError> {
    if check_if_logged(&app_data.jwt, &req) != LoggedStatus::LoggedIn {
        return Ok(HttpResponse::Forbidden().finish())
    }

    let mut recipe_name = String::new();
    let mut raw_image_bytes = Vec::new();

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
            x if x.starts_with('d') => {
                let mut new_image = Vec::new();
                while let Some(chunk) = field.try_next().await? {
                    new_image.extend_from_slice(&chunk);
                    if recipe_name.len() > crate::cdn::MAX_IMAGE_SIZE {
                        return Ok(HttpResponse::PayloadTooLarge().finish())
                    }
                }
                raw_image_bytes.push(new_image);
            },
            _ => return Ok(HttpResponse::BadRequest().finish())
        }
    }

    let result = app_data.cdn.transaction(|cdn| {
        for image in raw_image_bytes {
            cdn.upload_image(&recipe_name, &image)?
        }

        Ok(())
    });

    match result {
        Ok(()) => Ok(HttpResponse::Ok().finish()),
        Err(e) => Ok(e.into()),
    }
}

pub async fn upload_images(recipe_name: &str, images: &[File]) -> Result<Result<(), super::Error>, ServerFnError> {
    let location = window().unwrap().location();
    let url = format!(
        "{}//{}/cdn/img/upload_images",
        location.protocol().unwrap(),
        location.host().unwrap(),
    );

    let form_data = FormData::new().unwrap();
    form_data.append_with_str("r", recipe_name).unwrap();
    for file in images {
        form_data.append_with_blob("d[]", file).unwrap();
    }
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

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn delete_images(recipe_name: String, image_names: Vec<String>) -> Result<Result<(), Error>, ServerFnError> {
    let app_data = extract_app_data().await?;
    let request = leptos::expect_context::<actix_web::HttpRequest>();

    match check_if_logged(&app_data.jwt, &request) {
        LoggedStatus::LoggedOut => {
            Ok(Err(Error::Unauthorized))
        },
        LoggedStatus::LoggedIn => {
            let cdn = &app_data.cdn;
            let image_list = cdn.get_image_list(&recipe_name)?;
            { // Check if all image_names are in image_list
                use std::collections::HashSet;
                
                let set = image_list.iter().collect::<HashSet<_>>();
                if !image_names.iter().all(|i| set.contains(i)) {
                    return Err(CdnError::ImageDoesntExist.into());
                }
            }
            cdn.delete_images(&recipe_name, &image_names)?;

            Ok(Ok(()))
        },
    }
}
