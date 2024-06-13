use leptos::*;
use std::collections::BTreeSet;

use crate::api;
use crate::app::RecipeNamesContext;

use super::GoBack;

#[component]
pub fn DeleteRecipe() -> impl IntoView {
    let recipe_names = expect_context::<RwSignal<RecipeNamesContext>>();
    let selected_names = create_rw_signal(BTreeSet::<String>::new());

    let confirm_node = create_node_ref::<html::Input>();
    let confirm_signal = create_rw_signal(false);

    let delete_recipes = create_action(move |recipe_names: &Vec<String>| { 
        let recipe_names = recipe_names.clone();
        async move {
            api::recipes::delete_recipes(recipe_names).await
        }
    });
    let delete_result = delete_recipes.value();

    view! {
        <GoBack />
        <h2> "Delete recipes" </h2>
        <div style="max-height: 15rem; overflow-y: scroll;">
            {move || recipe_names.get().0.into_iter()
                .map(|n| {
                    let input_node = create_node_ref::<html::Input>();

                    view! {
                        <div>
                            <input
                                type="checkbox"
                                id=format!("delete-recipe-{n}")
                                name=&n
                                value=&n
                                node_ref=input_node
                                on:change=move |_| {
                                    let input_node = input_node.get().unwrap();
                                    let name = input_node.value();

                                    selected_names.update(move |x| {
                                        if input_node.checked() {
                                            x.insert(name);
                                        } else {
                                            x.remove(&name);
                                        }
                                    });
                                }
                            />
                            <label for=format!("delete-recipe-{n}")> {&n} </label>
                        </div>
                    }
                })
                .collect::<Vec<_>>()
            }
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
                selected_names.get().is_empty() || (!confirm_signal.get())
            }
            on:click=move |_| {
                let recipe_names: Vec<String> = selected_names.get_untracked().iter().map(|s| s.to_string()).collect();
                delete_recipes.dispatch(recipe_names);
            }
        > "Delete selected recipes" </button>
        {move || delete_result.get().map(|r| match r {
            Ok(Ok(())) => {
                recipe_names.update(|x| x.0.retain(|s| !selected_names.get_untracked().contains(s)));
                selected_names.update(|x| x.clear());
                confirm_node.get().unwrap().set_checked(false);
                confirm_signal.set(false);
                view! {<p> "Recipe deleted successfully" </p>}
            },
            Ok(Err(err)) => view! {<p style="color:red;"> {err.to_string()} </p>},
            Err(err) => view! {<p style="color:red;"> {format!("Error deleting a recipe:\n{err}")} </p>},
        })}
    }
}

