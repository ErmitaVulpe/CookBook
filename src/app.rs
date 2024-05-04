use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::ops::Deref;

use crate::api;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/cook-book.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    let action1 = create_action(|_: &()| api::auth::log_in(api::auth::UserRaw::new("username", "password")));

    let file_input = create_node_ref::<html::Input>();

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <button on:click=move |_| action1.dispatch(())>"Click Me: " {count}</button>
        <br/>
        <input
            name="Upload photos"
            type="file"
            node_ref=file_input
            accept="image/*"
            multiple

            on:change=move |ev| {
                let file_list: web_sys::FileList = file_input().unwrap().deref().files().unwrap();
                for i in 0..file_list.length() {
                    logging::log!("{:#?}", file_list.get(i).unwrap());
                }
                logging::log!("{:#?}", file_list);
            }
        />
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
