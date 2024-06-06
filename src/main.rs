#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use cook_book::{app::*, AppData, api};

    dotenv::dotenv().ok();
    
    let app_data = web::Data::new(AppData::default());

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    println!("listening on http://{}", &addr);

    let leptos_options = conf.leptos_options;
    let leptos_options_data = web::Data::new(leptos_options.clone());
    let site_root = leptos_options.site_root.to_owned();


    {
        use diesel::prelude::*;

        let mut conn = app_data.get_conn().unwrap();

        {
            #[derive(Debug, Clone, Queryable, Selectable)]
            #[diesel(table_name = cook_book::schema::recipes)]
            #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
            struct Recipe {
                #[allow(dead_code)]
                name: String,
                #[allow(dead_code)]
                instructions: String,
            }

            use cook_book::schema::recipes::dsl::*;
            let result = recipes
                .select(Recipe::as_select())
                .load::<Recipe>(&mut conn);
            println!("{:#?}", result);
        }
        {
            #[derive(Debug, Clone, Queryable, Selectable)]
            #[diesel(table_name = cook_book::schema::recipe_ingredients)]
            #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
            struct RecipeIngredient {
                #[allow(dead_code)]
                id: i32,
                #[allow(dead_code)]
                recipe_name: String,
                #[allow(dead_code)]
                ingredient_id: i32,
                #[allow(dead_code)]
                ammount: String,
            }

            use cook_book::schema::recipe_ingredients::dsl::*;
            let result = recipe_ingredients
                .select(RecipeIngredient::as_select())
                .load::<RecipeIngredient>(&mut conn);
            println!("{:#?}", result);
        }
    }

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
