use leptos::*;
use std::ops::Deref;

use crate::api::{
    self,
    Error,
    recipes::{Recipe, IngredientWithAmount},
};

use super::{GoBack, IngredientsContext, RecipeNamesContext};

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

    let create_recipe = create_action(move |recipe: &Recipe| { 
        let recipe = recipe.clone();
        async move {
            api::recipes::create_recipe(recipe).await
        }
    });

    let clear_form = move || {
        form_node.get().unwrap().deref().reset();
        selected_ingredients.update(|v| v.clear());
    };

    let disabled = create_recipe.pending();
    let result = create_recipe.value();

    view! {
        <GoBack />
        <h2> "Create a new recipe" </h2>
        <form
            node_ref=form_node
            on:submit=move |ev| {
                ev.prevent_default();

                let name = name_node.get_untracked().unwrap().deref().value();
                new_recipe_name.set(name.clone());

                let new_recipe = Recipe {
                    name,
                    instructions: instructions_node.get_untracked().unwrap().deref().value(),
                    ingredients: selected_ingredients.get_untracked(),
                };
                create_recipe.dispatch(new_recipe);
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
                type="submit"
                prop:disabled=move || disabled.get()
            >
                "Create recipe"
            </button>
            {move || result.get().map(|x| match x {
                Ok(Ok(())) => {
                    clear_form();
                    recipe_names.update(|r| r.0.push(new_recipe_name.get_untracked()));
                    view! {<p> "Recipe created successfully" </p>}
                },
                Ok(Err(err)) => view! {<p style="color:red;"> {match err {
                    Error::Unauthorized => "Session expired please refresh the site"
                }} </p>},
                Err(err) => view! {<p style="color:red;"> {format!("Error creating a recipe:\n{err}")} </p>},
            })}
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
                    selected=move || select_ingredient.get() == None
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

