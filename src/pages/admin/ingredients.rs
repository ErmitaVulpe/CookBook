use leptos::*;

use std::collections::BTreeSet;
use super::GoBack;
use crate::app::IngredientsContext;
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
    let ingredient_context = expect_context::<RwSignal<IngredientsContext>>();

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
                name.clone(),
                is_indexable,
            ).await;

            create_ingredient_message.set( match result {
                Err(err) => Err(err.to_string()),
                Ok(Err(err)) => Err(err.to_string()),
                Ok(Ok(val)) => match val {
                    CreateIngredientResult::IngredientExists => Err("Ingredient already exists".to_string()),
                    CreateIngredientResult::Ok => {
                        form_node.get_untracked().unwrap().reset();
                        ingredient_context.update(|v| {
                            let b_tree_map = &mut v.0;
                            let last_id = b_tree_map.keys().next_back().unwrap_or(&0);
                            let new_ingredient_id = last_id + 1;
                            let new_ingredient = api::recipes::Ingredient {
                                id: new_ingredient_id,
                                name,
                                is_indexable,
                            };
                            b_tree_map.insert(new_ingredient_id, new_ingredient);
                        });
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
                    name: name_node.get_untracked().unwrap().value().trim().to_string(),
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
    use api::ingredients::DeleteIngredientResult;

    let ingredient_context = expect_context::<RwSignal<IngredientsContext>>();
    let selected_ingredients = create_rw_signal(BTreeSet::<i32>::new());
    // Required for displaying the error msg
    let rqeuested_ingredients_to_delete = create_rw_signal(None);
    
    let confirm_node = create_node_ref::<html::Input>();
    let confirm_signal = create_rw_signal(false);

    #[derive(Clone)]
    enum DeleteMessage {
        Empty,
        Pending,
        Error(String),
        Result(Vec<DeleteIngredientResult>),
    }
    let delete_message = create_rw_signal(DeleteMessage::Empty);

    create_effect(move |_| {
        logging::log!("{:#?}", ingredient_context.get());
    });

    let delete_ingredients = create_action(move |()| {
        delete_message.set(DeleteMessage::Pending);
        let ingredients_to_delete = selected_ingredients
            .get_untracked()
            .iter()
            .map(|i| i.to_owned())
            .collect::<Vec<_>>();
        rqeuested_ingredients_to_delete.set(Some(ingredients_to_delete.clone()));
        let req = api::ingredients::delete_ingredients(ingredients_to_delete);
        async move {
            let msg = match req.await {
                Err(err) => DeleteMessage::Error(err.to_string()),
                Ok(Err(err)) => DeleteMessage::Error(err.to_string()),
                Ok(Ok(val)) => DeleteMessage::Result(val),
            };
            delete_message.set(msg);
        }
    });

    view! {
        <GoBack />
        <h2> "Delete ingredients" </h2>
        <div style="max-height: 15rem; overflow-y: scroll;">
            <For
                each=move || ingredient_context.get().0
                key=|ingredient| ingredient.0
                children=move |(id, ingredient)| {
                    let input_node = create_node_ref::<html::Input>();
                    let ingredient_name = ingredient.name.replace(' ', "-");

                    view! {
                        <div>
                            <input
                                type="checkbox"
                                id=format!("delete-recipe-{ingredient_name}")
                                name=&ingredient_name
                                value=id
                                node_ref=input_node
                                on:change=move |_| {
                                    let input_node = input_node.get().unwrap();
                                    let name = input_node.value().parse().unwrap();

                                    selected_ingredients.update(move |x| {
                                        if input_node.checked() {
                                            x.insert(name);
                                        } else {
                                            x.remove(&name);
                                        }
                                    });
                                }
                            />
                            <label for=format!("delete-recipe-{ingredient_name}")> {&ingredient.name} </label>
                        </div>
                    }
                }
            />
        </div>
        <div style="padding: 1rem 0;">
            <input
                type="checkbox"
                id="confirm-delete"
                node_ref=confirm_node
                on:change=move |_| {
                    confirm_signal.set( confirm_node.get().unwrap().checked() );
                }
            />
            <label for="confirm-delete"> "Yes im sure" </label>
        </div>
        <button
            disabled=move || {
                selected_ingredients.get().is_empty() || (!confirm_signal.get())
            }
            on:click=move |_| {
                delete_ingredients.dispatch(());
            }
        > "Delete selected recipes" </button>
        {move || match delete_message.get() {
            DeleteMessage::Empty => ().into_view(),
            DeleteMessage::Pending => "Uploading".into_view(),
            DeleteMessage::Error(err) => err.into_view(),
            DeleteMessage::Result(val) => {
                let rqeuested_ingredients_to_delete = rqeuested_ingredients_to_delete.get_untracked().unwrap();

                let ok_count = val.iter()
                    .enumerate()
                    .filter(|(_, r)| **r == DeleteIngredientResult::Ok )
                    .map(|(i, _)| {
                        let ingredient_id = rqeuested_ingredients_to_delete[i];
                        selected_ingredients.update(|s| { s.remove(&ingredient_id); });
                        ingredient_context.update(|s| { s.0.remove(&ingredient_id); });
                    })
                    .count();
                let mut err_iter = val.iter()
                    .enumerate()
                    .filter_map(|(i, r)| {
                        if let DeleteIngredientResult::IngredientInUse(recipes) = r {
                            Some((i, recipes))
                        } else {
                            None
                        }
                    })
                    .peekable();
            
                view! {
                    <p> {format!("Successfully deleted {ok_count} ingredients")} </p>
                    {match err_iter.peek() {
                        None => ().into_view(),
                        Some(_) => {
                            let ingredient_context = ingredient_context.get_untracked();
                            view! {
                                <p> "Couldn't delete the following ingredients because they are used in these recipes" </p>
                                {err_iter.map(|(i, recipes)| view! {
                                    <li>{
                                        let ingredient_id = rqeuested_ingredients_to_delete[i];
                                        format!(
                                            "{}: {}",
                                            ingredient_context.0[&ingredient_id].name,
                                            recipes.join(", "),
                                        )
                                    }</li>
                                }).collect_view()}
                            }.into_view()
                        },
                    }}
                }.into_view()
            },
        }}
    }
}
