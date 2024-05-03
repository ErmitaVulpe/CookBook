use serde_repr::*;

#[cfg(feature="ssr")]
use actix_web::{get, HttpResponse, Responder, web};

// #[cfg(feature="ssr")]
// pub type cdn_meta = std::collections::HashMap<(u32, u32), FileExtensions>;

#[cfg(feature="ssr")]
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::scope("").service(get_img))
    ;
}

#[cfg(feature="ssr")]
#[get("/{folder}/{img}")]
async fn get_img(
    _app_data: web::Data<crate::AppData>,
    _path: web::Path<(u32, u32)>,
) -> impl Responder {
    // let (folder, img) = path.into_inner();
    // match NamedFile::open(format!("{}{folder}/{img}", app_data.cdn_path)) {
    //     Ok(val) => 
    // }
    // NamedFile::open(format!("{}{folder}/{img}", app_data.cdn_path))
    //     .set_content_type(mimw::Mime::from_str("text/plain").unwrap())
    HttpResponse::Ok().body("Hello world!")
}

// #[derive(Debug, Serialize_repr, Deserialize_repr)]
// #[repr(u8)]
// pub enum FileExtensions {
//     Jpg,
//     Png,
//     Webp,
// }

// impl TryFrom<&str> for FileExtensions {
//     type Error = ();
    
//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         Ok(match value {
//             "jpg" | "jpeg" => Self::Jpg,
//             "png" => Self::Png,
//             "webp" => Self::Webp,
//             _ => return Err(()),
//         })
//     }
// }

// impl Into<&str> for FileExtensions {
//     fn into(self) -> &'static str {
//         match self {
//             Self::Jpg => "jpg",
//             Self::Png => "png",
//             Self::Webp => "webp",
//         }
//     }
// }
