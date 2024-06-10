use leptos::*;
use web_sys::File;
use std::ops::Deref;

use crate::api::{
    self,
    Error,
    recipes::{Recipe, IngredientWithAmount},
};
use crate::app::{IngredientsContext, RecipeNamesContext};

use super::GoBack;

#[derive(Clone, Debug)]
struct NewRecipeData {
    recipe: Recipe,
    icon: File,
}

#[component]
pub fn CreateRecipe() -> impl IntoView {
    let form_node = create_node_ref::<html::Form>();
    let name_node = create_node_ref::<html::Input>();
    let instructions_node = create_node_ref::<html::Textarea>();
    let icon_node = create_node_ref::<html::Input>();

    let ingredients = expect_context::<RwSignal<IngredientsContext>>();
    let recipe_names = expect_context::<RwSignal<RecipeNamesContext>>();

    let selected_ingredients = create_rw_signal(Vec::new());
    let new_recipe_name = create_rw_signal(String::new());

    let create_recipe_message = create_rw_signal(Ok::<String, String>(String::new()));

    let clear_form = move || {
        form_node.get_untracked().unwrap().deref().reset();
        selected_ingredients.update(|v| v.clear());
    };

    let create_recipe = create_action(move |recipe: &NewRecipeData| { 
        let recipe = recipe.clone();
        let NewRecipeData {
            recipe,
            icon,
        } = recipe;
        let recipe_name = recipe.name.clone();
        async move {
            create_recipe_message.set(Ok("Uploading".to_string()));

            match api::recipes::create_recipe(recipe).await {
                Err(err) => {
                    create_recipe_message.set(Err(format!("Error creating a recipe: {err}")));
                    return;
                },
                Ok(Err(err)) => {
                    create_recipe_message.set(match err {
                        Error::Unauthorized => Err("Session expired please refresh the site".to_string()),
                    });
                    return;
                },
                Ok(Ok(())) => {},
            }

            let result = api::img::upload_icon(&recipe_name, &icon).await;
            match result {
                Err(err) => {
                    create_recipe_message.set(Err(format!("Error creating a recipe: {err}")));
                    return;
                },
                Ok(Err(err)) => {
                    create_recipe_message.set(match err {
                        Error::Unauthorized => Err("Session expired please refresh the site".to_string()),
                    });
                    return;
                },
                Ok(Ok(())) => {
                    clear_form();
                    recipe_names.update(|r| r.0.push(new_recipe_name.get_untracked()));
                },
            }

            create_recipe_message.set(Ok("Recipe created successfully".to_string()));
        }
    });

    let disabled = create_recipe.pending();

    view! {
        <GoBack />
        <h2> "Create a new recipe" </h2>
        <form
            node_ref=form_node
            on:submit=move |ev| {
                ev.prevent_default();

                let name = name_node.get_untracked().unwrap().deref().value();
                new_recipe_name.set(name.clone());

                let recipe = Recipe {
                    name,
                    instructions: instructions_node.get_untracked().unwrap().deref().value(),
                    ingredients: selected_ingredients.get_untracked(),
                };

                let recipe_data = NewRecipeData {
                    recipe,
                    icon: icon_node.get().unwrap().files().unwrap().get(0).unwrap(),
                };

                create_recipe.dispatch(recipe_data);
            }
        >
            <div>
                <input
                    type="text"
                    name="name"
                    node_ref=name_node
                    placeholder="Recipe name"
                    prop:disabled=move || disabled.get()
                    required
                />
            </div>
            <RecipeFormIngredientSelector
                ingredients=ingredients
                selected_ingredients=selected_ingredients
                disabled=disabled
            />
            <div>
                <h5> "Instructions" </h5>
                <textarea
                    node_ref=instructions_node
                    prop:disabled=move || disabled.get()
                    cols=50
                    rows=15
                ></textarea>
            </div>
            <div>
                <h5> "Recipe icon" </h5>
                <input
                    type="file"
                    name="Upload icon"
                    node_ref=icon_node
                    prop:disabled=move || disabled.get()
                    accept="image/*"
                    required
                />
            </div>
            <button
                type="button"
                on:click=|_| {
                    web_sys::window().unwrap().open_with_url_and_target_and_features(
                        "/admin/create_recipe/preview",
                        "_blank",
                        "popup"
                    ).unwrap();
                }
            > "Preview" </button>
            <button
                type="submit"
                prop:disabled=move || disabled.get()
            >
                "Create recipe"
            </button>
            {move || match create_recipe_message.get() {
                Ok(val) => view! {<p> {val} </p>},
                Err(err) => view! {<p style="color:red;"> {err} </p>},
            }}
        </form>
    }
}

#[component]
fn RecipeFormIngredientSelector(
    #[prop(into)]
    ingredients: Signal<IngredientsContext>,
    selected_ingredients: RwSignal<Vec<IngredientWithAmount>>,
    #[prop(into)]
    disabled: Signal<bool>,
) -> impl IntoView {
    let select_ingredient = create_rw_signal(None::<i32>);
    let derived_ingredients = Signal::derive(move || {
        ingredients.get().0.values().cloned().collect::<Vec<_>>()
    });
    let ingredient_ammount = create_node_ref::<html::Input>();

    view! {
        <h5> "Add ingredients" </h5>
        <div>
            <select prop:disabled=move || disabled.get() on:change=move |ev| {
                let new_value = event_target_value(&ev).parse::<i32>().unwrap();
                select_ingredient.set({
                    if new_value == -1 {
                        None
                    } else {
                        Some(new_value)
                    }
                });
            }>
                <option
                    value=-1
                    selected=move || select_ingredient.get().is_none()
                > "-" </option>
                <For
                    each=derived_ingredients
                    key=|ingredient| ingredient.id
                    children=move |ingredient| view! {
                        <option
                            value=ingredient.id
                            selected=move || select_ingredient.get() == Some(ingredient.id)
                        >
                            {ingredients.get().0.get(&ingredient.id).unwrap().name.to_string()}
                        </option>
                    }
                />
            </select>
            <input
                type="text"
                name="Ingredient ammount"
                node_ref=ingredient_ammount
                prop:disabled=move || disabled.get()
            />
            <input
                type="button"
                value="Add ingredient"
                prop:disabled=move || disabled.get()
                on:click=move |_| {
                    selected_ingredients.update(move |i| {
                        let selected_id = select_ingredient.get_untracked();
                        if let Some(val) = selected_id {
                            if !i.iter().any(|x| x.ingredient_id == val) {
                                i.push(IngredientWithAmount {
                                    ingredient_id: val,
                                    ammount: ingredient_ammount.get().unwrap().deref().value()
                                });
                            }
                        }
                    })
                }
            />
        </div>
        <ul>
            <For
                each=selected_ingredients
                key=|ingredient| ingredient.ingredient_id
                children=move |ingredient_with_ammount| {
                    view! {
                        <li>
                            <button
                                style="color:red"
                                prop:disabled=move || disabled.get()
                                on:click=move |_| {
                                    selected_ingredients.update(|counters| {
                                        counters.retain(|x| x.ingredient_id != ingredient_with_ammount.ingredient_id)
                                    });
                                }
                            > "Remove" </button>
                            <span style="padding-left:1em">{
                                format!(
                                    "{}: {}",
                                    ingredients.get().0.get(&ingredient_with_ammount.ingredient_id).unwrap().name,
                                    ingredient_with_ammount.ammount,
                                )
                            }</span>
                        </li>
                    }
                }
            />
        </ul>
    }
}

#[component]
pub fn PreviewNewRecipe(

) -> impl IntoView {
    if cfg!(feature = "ssr") {
        ().into_view()
    } else {
        logging::log!("{:#?}", window().opener());
        "TODO".into_view()
    }
}
