use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use std::collections::BTreeMap;

use crate::{
    api::recipes::{get_ingredients, Ingredient},
    pages::*,
};


#[derive(Clone, Debug)]
pub struct IngredientsContext(pub BTreeMap<i32, Ingredient>);
#[derive(Clone, Debug)]
pub struct RecipeNamesContext(pub Vec<String>);

#[derive(Clone, Debug)]
pub struct BoundaryError(pub String);

impl BoundaryError {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl std::error::Error for BoundaryError {}
impl std::fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/cook-book.css"/>
        <Title text="Cook book"/>

        <Router>
            <nav>
                <A href="/"> "To home" </A>
            </nav>
            <Routes>
                <Route path="" view=home::Home/>
                <Route path="/r" view=PreleadResources>
                    <Route path="/:recipe_name" view=view_recipe::ViewRecipe />
                </Route>
                <Route path="/admin" view=admin::Admin>
                    <Route path="" view=admin::ToolList/>
                    <Route path="/create_recipe" view=admin::create_recipe::CreateRecipe/>
                    <Route path="/upload_image" view=admin::images::upload_image::UploadImage/>
                    <Route path="/delete_image" view=admin::images::delete_image::DeleteImage/>
                    <Route path="/delete_recipe" view=admin::delete_recipe::DeleteRecipe/>
                    <Route path="/create_ingredient" view=admin::ingredients::CreateIngredient/>
                    <Route path="/delete_ingredient" view=admin::ingredients::DeleteIngredient/>
                </Route>
                <Route path="/admin/create_recipe/preview" view=admin::create_recipe::PreviewNewRecipe/>
                <Route path="/*any" view=not_found::NotFound/>
            </Routes>
        </Router>
    }
}

#[component]
fn PreleadResources() -> impl IntoView {
    let ingredients = create_resource(
        || (),
        |_| get_ingredients(),
    );

    let view = move || {
        let result = move || ingredients.get()
            .ok_or(BoundaryError::new("Error loading ingredients".to_string()));

        result().map(|ingredients| {
            match ingredients {
                Ok(ingredients) => {
                    let mut map = BTreeMap::new();
                    for i in ingredients {
                        map.insert(i.id, i);
                    }
                    provide_context(IngredientsContext(map));
                    Ok(view! {
                        <Outlet />
                    })
                },
                Err(err) => Err(BoundaryError::new(err.to_string())),
            }
        })
    };

    view! {
        <Suspense fallback=|| ()>
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
                {view}
            </ErrorBoundary>
        </Suspense>
    }
}
