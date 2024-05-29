use leptos::*;

use crate::api::{self, Error};

use super::{GoBack, RecipeNamesContext};

#[component]
pub fn UploadImage() -> impl IntoView {
    let recipe_names = expect_context::<RwSignal<RecipeNamesContext>>();
    let selected_recipe = create_rw_signal(None::<String>);
    let upload_message = create_rw_signal(Ok::<String, String>(String::new()));

    let form_node = create_node_ref::<html::Form>();
    let images_node = create_node_ref::<html::Input>();

    let clear_form = move || {
        form_node.get_untracked().unwrap().reset();
        selected_recipe.set(None);
    };

    let upload_images = create_action(move |_: &()| {
        upload_message.set(Ok("Uploading".to_string()));

        let recipe_name = selected_recipe.get_untracked().unwrap();
        async move {
            let raw_files = images_node.get_untracked().unwrap().files().unwrap();
            let n_of_files = raw_files.length();

            let mut file_list = Vec::with_capacity(n_of_files as usize);
            for i in 0..n_of_files {
                file_list.push(raw_files.get(i).unwrap());
            }

            let result = api::img::upload_images(
                &recipe_name,
                &file_list,
            ).await;

            match result {
                Err(err) => {
                    upload_message.set(Err(format!("Error creating a recipe: {err}")));
                    return;
                },
                Ok(Err(err)) => {
                    upload_message.set(match err {
                        Error::Unauthorized => Err("Session expired please refresh the site".to_string()),
                    });
                    return;
                },
                Ok(Ok(())) => clear_form(),
            }

            clear_form();
            upload_message.set(Ok("Images uploaded successfully".to_string()));
        }
    });

    let disabled = upload_images.pending();

    view! {
        <GoBack />
        <h2> "Upload images" </h2>
        <form
            node_ref=form_node
            on:submit=move |ev| {
                ev.prevent_default();
                upload_images.dispatch(());
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
            <h5> "Select images to upoload" </h5>
            <div>
                <input
                    type="file"
                    name="Upload icon"
                    node_ref=images_node
                    prop:disabled=move || disabled.get() || selected_recipe.get().is_none()
                    accept="image/*"
                    multiple
                    required
                />
            </div>
            <button
                type="submit"
                prop:disabled=move || disabled.get() || selected_recipe.get().is_none()
            >
                "Upload images"
            </button>
            {move || match upload_message.get() {
                Ok(val) => view! {<p> {val} </p>},
                Err(err) => view! {<p style="color:red;"> {err} </p>},
            }}
        </form>
    }
}
