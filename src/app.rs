use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/cook-book.css"/>
        <Title text="Welcome to Leptos"/>

        <Router>
            <main>
                <Routes>
                    <Route path="" view=home::Home/>
                    <Route path="/admin" view=admin::Admin>
                        <Route path="" view=admin::ToolList/>
                        <Route path="/create_recipe" view=admin::create_recipe::CreateRecipe/>
                        <Route path="/delete_recipe" view=admin::delete_recipe::DeleteRecipe/>
                    </Route>
                    <Route path="/*any" view=not_found::NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

