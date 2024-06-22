use leptos::*;
use leptos_router::*;
use crate::api;

#[component]
pub fn Home() -> impl IntoView {
    let recipes = create_resource(
        ||(),
        |_| api::recipes::get_recipe_names(),
    );

    view! {
        <Suspense fallback=move || view! { <p>"Loading"</p> }>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <div class="error">
                        <h1>"Something went wrong."</h1>
                        <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, error)| view! { <li>{error.to_string()} </li> })
                            .collect_view()
                        }
                        </ul>
                    </div>
                }
            }>
                <h1> "Recipes:" </h1>
                <ul>
                    {move || {
                        recipes.get().map(|r| r.map(|v| 
                            v.into_iter().map(|recipe_name| view! {
                                <li>
                                    <A
                                        href=format!("/r/{recipe_name}")
                                    > {recipe_name} </A>
                                </li>
                            }).collect_view()
                        ))
                    }}
                </ul>
            </ErrorBoundary>
        </Suspense>
    }
}
