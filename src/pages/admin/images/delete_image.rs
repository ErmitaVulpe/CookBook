use leptos::*;
use std::{
    collections::HashSet,
    sync::Arc,
};

use crate::api::{self, Error};
use super::{GoBack, RecipeNamesContext};

#[component]
pub fn DeleteImage() -> impl IntoView {
    let recipe_names = expect_context::<RwSignal<RecipeNamesContext>>();
    let selected_recipe = create_rw_signal(None::<String>);
    let selected_images = create_rw_signal(HashSet::new());
    let delete_message = create_rw_signal(Ok::<String, String>(String::new()));

    let form_node = create_node_ref::<html::Form>();

    let clear_form = move || {
        form_node.get_untracked().unwrap().reset();
        selected_recipe.set(None);
    };

    let delete_images = create_action(move |image_list: &Vec<String>| {
        delete_message.set(Ok("Deleting".to_string()));

        let recipe_name = selected_recipe.get_untracked().unwrap();
        let image_list = image_list.clone();
        async move {

            let result = api::img::delete_images(
                recipe_name,
                image_list,
            ).await;

            match result {
                Err(err) => {
                    delete_message.set(Err(format!("Error deleting images: {err}")));
                    return;
                },
                Ok(Err(err)) => {
                    delete_message.set(match err {
                        Error::Unauthorized => Err("Session expired please refresh the site".to_string()),
                    });
                    return;
                },
                Ok(Ok(())) => {},
            }

            clear_form();
            delete_message.set(Ok("Images deleted successfully".to_string()));
        }
    });

    let disabled = delete_images.pending();

    let image_list = create_resource(
        selected_recipe,
        |selected_recipe| async move {
            match selected_recipe {
                None => Ok(Vec::new()),
                Some(name) => api::recipes::get_images_for_recipe(name).await,
            }
        },
    );

    view! {
        <GoBack />
        <h2> "Delete images" </h2>
        <form
            node_ref=form_node
            on:submit=move |ev| {
                ev.prevent_default();
                let owned_image_list = selected_images
                    .get_untracked()
                    .iter()
                    .map(|x: &String| x.to_owned())
                    .collect::<Vec<_>>();
                delete_images.dispatch(owned_image_list);
            }
        >
            <h5> "Select recipe" </h5>
            <div>
                <super::SelectRecipe
                    recipe_list = recipe_names
                    selected_recipe = selected_recipe
                    disabled = disabled
                />
            </div>
            <Suspense
                fallback=move || view! { <p>"Loading..."</p> }
            >
                {move || image_list.get().map(|image_list| {
                    match image_list {
                        Err(err) => view! { <p style="color:red;"> {err.to_string()} </p> }.into_view(),
                        Ok(mut image_list) => {
                            if image_list.is_empty() || selected_recipe.get().is_none() {
                                return ().into_view();
                            }

                            image_list.sort();
                            let recipe_name = selected_recipe.get().unwrap();
                            selected_images.update(|m| m.clear());

                            view! {
                                <h5> "Select images to delete" </h5>
                                <div style="max-height: 15rem; overflow-y: scroll;">
                                    {move || {
                                        let window = web_sys::window().unwrap();
                                        let location = window.location();
                                        let url_prefix = Arc::new(format!(
                                            "{}//{}/cdn/img/get/{}/",
                                            location.protocol().unwrap(),
                                            location.host().unwrap(),
                                            recipe_name
                                        ));

                                        image_list.iter().map(move |image_name| {
                                            let input_node = create_node_ref::<html::Input>();
                                            let url_prefix = Arc::clone(&url_prefix);
                                            let preview_url = format!(
                                                "{}{}",
                                                url_prefix.to_lowercase(),
                                                image_name,
                                            );

                                            view! {
                                                <div>
                                                    <input
                                                        type="checkbox"
                                                        id=format!("delete-image-{image_name}")
                                                        name=image_name
                                                        value=image_name
                                                        node_ref=input_node
                                                        on:change=move |_| {
                                                            let input_node = input_node.get().unwrap();
                                                            let name = input_node.value();

                                                            selected_images.update(move |x| {
                                                                if input_node.checked() {
                                                                    x.insert(name);
                                                                } else {
                                                                    x.remove(&name);
                                                                }
                                                            });
                                                        }
                                                    />
                                                    <button
                                                        type="button"
                                                        on:click=move |_| {
                                                            web_sys::window().unwrap().open_with_url_and_target(
                                                                &preview_url,
                                                                "_blank",
                                                            ).unwrap();
                                                        }
                                                    > "Preview" </button>
                                                    <label
                                                        style="padding-left: 0.5rem;"
                                                        for=format!("delete-image-{image_name}")
                                                    > {image_name} </label>
                                                </div>
                                            }
                                        }
                                    ).collect_view()}}
                                </div>
                            }.into_view()
                        },
                    }
                })}
            </Suspense>
            <button
                type="submit"
                prop:disabled=move || disabled.get() || selected_recipe.get().is_none() || selected_images.get().is_empty()
            >
                "Delete images"
            </button>
            {move || match delete_message.get() {
                Ok(val) => view! {<p> {val} </p>},
                Err(err) => view! {<p style="color:red;"> {err} </p>},
            }}
        </form>
    }
}
