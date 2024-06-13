pub mod auth;
pub mod img;
pub mod ingredients;
pub mod recipes;

use serde_repr::*;
use std::fmt;

#[cfg(feature="ssr")]
use actix_web::web;

#[cfg(feature="ssr")]
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::scope("/img").configure(img::api))
    ;
}

#[cfg(feature="ssr")]
async fn extract_app_data() -> Result<std::sync::Arc<crate::AppData>, leptos::ServerFnError> {
    leptos_actix::extract::<actix_web::web::Data<crate::AppData>>()
        .await.map(|i| i.into_inner())
}

#[repr(u8)]
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
pub enum Error {
    Unauthorized,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Error::Unauthorized => "Session expired please refresh the site",
        })
    }
}

pub fn is_valid_recipe_name(recipe_name: &str) -> bool {
    recipe_name.chars().all(|c|
        c.is_alphanumeric() ||
        c.is_whitespace()
    )
}
