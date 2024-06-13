use leptos::*;

use super::GoBack;
use crate::api::{
    self,
    ingredients::CreateIngredientResult,
};

#[component]
pub fn CreateIngredient() -> impl IntoView {
    let form_node = create_node_ref::<html::Form>();
    let name_node = create_node_ref::<html::Input>();
    let checkbox_node = create_node_ref::<html::Input>();
    let is_indexable = create_rw_signal(false);

    let create_ingredient_message = create_rw_signal(Ok::<String, String>(String::new()));

    #[derive(Clone)]
    struct NewIngredient {
        name: String,
        is_indexable: bool,
    }

    let action = create_action(move |new_ingredient: &NewIngredient| {
        create_ingredient_message.set(Ok("Uploading".to_string()));
        let NewIngredient { name, is_indexable } = new_ingredient.clone();
        async move {
            let result = api::ingredients::create_ingredient(
                name,
                is_indexable,
            ).await;

            create_ingredient_message.set( match result {
                Err(err) => Err(err.to_string()),
                Ok(Err(err)) => Err(err.to_string()),
                Ok(Ok(val)) => match val {
                    CreateIngredientResult::IngredientExists => Err("Ingredient already exists".to_string()),
                    CreateIngredientResult::Ok => {
                        form_node.get_untracked().unwrap().reset();
                        Ok("Recipe created successfully".to_string())
                    },
                }
            })
        }
    });
    let disabled = action.pending();
    
    view! {
        <GoBack />
        <h2> "Create a new ingredient" </h2>
        <form
            node_ref=form_node
            on:submit=move |ev| {
                ev.prevent_default();

                let new_ingredient = NewIngredient {
                    name: name_node.get_untracked().unwrap().value(),
                    is_indexable: is_indexable.get_untracked(),
                };
                action.dispatch(new_ingredient);
            }
        >
            <div>
                <input
                    type="text"
                    name="name"
                    id="new-ingredient-name"
                    node_ref=name_node
                    placeholder="Recipe name"
                    prop:disabled=move || disabled.get()
                    autocomplete="off"
                    required
                />
            </div>
            <div>
                <label
                    for="new-ingredient-is-indexable"
                > "Is indexable?" </label>
                <input
                    type="checkbox"
                    name="is indexable"
                    id="new-ingredient-is-indexable"
                    node_ref=checkbox_node
                    prop:disabled=move || disabled.get()
                    on:change=move |_| {
                        is_indexable.set(
                            checkbox_node.get_untracked().unwrap().checked()
                        );
                    }
                />
            </div>
            <div>
                <button type="submit"> "Create ingredient" </button>
            </div>
            {move || match create_ingredient_message.get() {
                Ok(val) => view! {<p> {val} </p>},
                Err(err) => view! {<p style="color:red;"> {err} </p>},
            }}
        </form>
    }
}

#[component]
pub fn DeleteIngredient() -> impl IntoView {
    
    view! {
        <GoBack />
        <h2> "Delete ingredients" </h2>
    }
}
