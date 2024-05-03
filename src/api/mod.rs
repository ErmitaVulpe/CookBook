pub mod auth;
pub mod img;

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
