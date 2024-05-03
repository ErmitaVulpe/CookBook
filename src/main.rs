#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use cook_book::{app::*, AppData, api};

    dotenv::dotenv().ok();

    // { // generate example meta
    //     use std::collections::HashMap;
    //     use std::fs::File;
    //     use std::io::Write;

    //     let mut map = HashMap::new();
    //     map.insert(0u32, 1u32);
    //     map.insert(1u32, 1u32);
    //     let map = map;

    //     let mut file = File::create("target/cdn/0/meta.cbor").unwrap();
    //     ciborium::ser::into_writer(&(2u32, &map), &mut file);
    //     let mut file = File::create("target/cdn/1/meta.cbor").unwrap();
    //     ciborium::ser::into_writer(&(2u32, &map), &mut file);
    // }
    
    let app_data = web::Data::new(AppData::new());
    {
        let a = app_data.clone().into_inner().cdn.add_photo_entry(2, cook_book::cdn::FileExtensions::Webp).unwrap();
        println!("{:#?}", app_data.clone().into_inner().cdn);
    }

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    println!("listening on http://{}", &addr);

    let leptos_options = conf.leptos_options;
    let leptos_options_data = web::Data::new(leptos_options.clone());
    let site_root = leptos_options.site_root.to_owned();

    HttpServer::new(move || {
        App::new()
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", &site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .service(web::scope("/cdn").configure(api::api))
            .leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)
            .app_data(leptos_options_data.clone())
            .app_data(app_data.clone())
        //.wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use cook_book::app::*;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
