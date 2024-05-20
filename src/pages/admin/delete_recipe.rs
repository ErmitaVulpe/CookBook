use leptos::*;
use std::collections::BTreeSet;

use super::{GoBack, RecipeNamesContext};

#[component]
pub fn DeleteRecipe() -> impl IntoView {
    let recipe_names = expect_context::<RwSignal<RecipeNamesContext>>();
    let selected_names = create_rw_signal(BTreeSet::<&str>::new());
    // TODO continue

    view! {
        <GoBack />
        <h2> "Delete recipes" </h2>
        <div style="max-height: 15rem; overflow-y: scroll;">
            {move || recipe_names.get().0.into_iter()
                .map(|n| view! { 
                    <div>
                        <input type="checkbox" id=format!("delete-recipe-{n}") name=&n />
                        <label for=format!("delete-recipe-{n}")> {&n} </label>
                    </div>
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

