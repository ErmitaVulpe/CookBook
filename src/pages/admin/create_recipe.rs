use leptos::*;
use web_sys::File;
use std::ops::Deref;

use crate::api::{
    self,
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
            if !api::is_valid_recipe_name(&recipe.name) {
                create_recipe_message.set(Ok("Invalid recipe name".to_string()));
                return;
            }
            create_recipe_message.set(Ok("Uploading".to_string()));

            match api::recipes::create_recipe(recipe).await {
                Err(err) => {
                    create_recipe_message.set(Err(format!("Error creating a recipe: {err}")));
                    return;
                },
                Ok(Err(err)) => {
                    create_recipe_message.set(Err(err.to_string()));
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
                    create_recipe_message.set(Err(err.to_string()));
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
                    id="new-recipe-name"
                    node_ref=name_node
                    placeholder="Recipe name"
                    prop:disabled=move || disabled.get()
                    autocomplete="off"
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
                    name="instructions"
                    id="new-recipe-instructions"
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
                    id="new-recipe-icon"
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
            <select
                name="select ingredient"
                prop:disabled=move || disabled.get()
                on:change=move |ev| {
                    let new_value = event_target_value(&ev).parse::<i32>().unwrap();
                    select_ingredient.set({
                        if new_value == -1 {
                            None
                        } else {
                            Some(new_value)
                        }
                    });
                }
            >
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
                            let ammount = ingredient_ammount.get().unwrap().deref().value();
                            if ammount.is_empty() {
                                return;
                            }

                            // Check if this ingredient already exist
                            if !i.iter().any(|x| x.ingredient_id == val) {
                                i.push(IngredientWithAmount {
                                    ingredient_id: val,
                                    ammount,
                                });
                            }
                        }
                    })
                }
            />
        </div>
        <ul id="new-recipe-ingredient-list">
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
pub fn PreviewNewRecipe() -> impl IntoView {
    let force_render_locally = create_local_resource(
        ||(),
        |_| async {},
    );

    fn render() -> Option<View> {
        let opener = if let Ok(opener) = window().opener() {
            opener
        } else {
            return Some(view! { <h1> "Hey man, don't do that" </h1> }.into_view());
        };
        let document = web_sys::Window::from(opener).document()?;

        use wasm_bindgen::{JsCast as _, JsValue};
        use web_sys::{FileReader, HtmlInputElement, HtmlTextAreaElement};

        let recipe_name = {
            let input_elem_raw = document.get_element_by_id("new-recipe-name")?;
            let input_elem_js_value = input_elem_raw.dyn_into::<JsValue>().ok()?;
            HtmlInputElement::from(input_elem_js_value).value()
        };

        let ingredient_list = {
            let input_elem_raw = document.get_element_by_id("new-recipe-ingredient-list")?;
            let li_items = input_elem_raw.get_elements_by_tag_name("li");
            let li_items_len: u32 = li_items.length();
            let mut ingredients_texts = Vec::with_capacity(li_items_len as usize);
            for i in 0..li_items_len {
                let li_string = li_items.get_with_index(i)?
                    .get_elements_by_tag_name("span")
                    .get_with_index(0)?
                    .text_content()?;
                ingredients_texts.push(li_string);
            }
            ingredients_texts
        };

        let instructions = {
            let text_area_elem_raw = document.get_element_by_id("new-recipe-instructions")?;
            let input_elem_js_value = text_area_elem_raw.dyn_into::<JsValue>().ok()?;
            HtmlTextAreaElement::from(input_elem_js_value).value()
        };

        {
            let input_elem_raw = document.get_element_by_id("new-recipe-icon")?;
            let input_elem_js_value = input_elem_raw.dyn_into::<JsValue>().ok()?;
            let input_elem = HtmlInputElement::from(input_elem_js_value);
            if let Some(val) = input_elem.files().unwrap().get(0) {
                let file_reader = FileReader::new().ok()?;
                file_reader.set_onload(Some(&web_sys::js_sys::Function::new_with_args(
                    "e",
                    "document.getElementById('icon-preview').src = e.target.result;",
                )));
                file_reader.read_as_data_url(&val).ok()?;
            }
        }

        Some(view! {
            <h1>{ &recipe_name }</h1>
            <img id="icon-preview" src="" alt="Icon Preview" />
            <h5> "Ingredients:" </h5>
            <ul>{
                ingredient_list.iter()
                    .map(|e| view! {<li>{e}</li>})
                    .collect_view()
            }</ul>
            <h5> "Instruction:" </h5>
            {
                use crate::md_parser::{deafult_options, parse};
                parse(&instructions, deafult_options())
            }
        }.into_view())
    }

    view! {
        <Show
            when=move || force_render_locally.get().is_some()
        >{ render().unwrap_or(view! {
            <h1> "Rendering error" </h1>
        }.into_view()) }</Show>
    }
}
