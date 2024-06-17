use std::sync::Arc;

use leptos::*;
use leptos_router::*;
use leptos_meta::*;

use crate::{
    api::{self, recipes::Recipe},
    app::{BoundaryError, IngredientsContext},
};

#[derive(Params, PartialEq)]
struct ViewRecipeParams {
    recipe_name: String,
}

#[component]
pub fn ViewRecipe() -> impl IntoView {
    let url_params = use_params::<ViewRecipeParams>();
    let recipe_name = Signal::derive( move || {
        url_params.with(|params| {
            params.as_ref()
                .map(|params| params.recipe_name.clone().replace('_', " "))
                .unwrap_or_default()
        })
    });


    let recipe_data = create_resource(
        recipe_name,
        move |recipe_name| async {
            Ok((
                api::recipes::get_recipe(recipe_name.clone()).await?,
                api::recipes::get_images_for_recipe(recipe_name).await?
            ))
        },
    );

    view! {
        <Suspense
            fallback=move || view! { <p>"Loading..."</p> }
        >{move || {
            recipe_data.get().map(|recipe_data: Result<(Option<Recipe>, Vec<String>), ServerFnError>| {
                match recipe_data {
                    Err(err) => view! {
                        <p style="color:red;"> {format!("Error loading recipe: {err}")} </p>
                    }.into_view(),
                    Ok((None, _)) => view! {
                        <super::not_found::NotFound /> // TODO figure out why it doesnt return 404
                    },
                    Ok((Some(recipe_data), images)) => {
                        view! {
                            <ViewRecipeComponent
                                recipe_data=recipe_data
                                image_list=images
                            />
                        }
                    }
                }
            })
        }}</Suspense>
    }
}

#[component]
pub fn ViewRecipeComponent(
    recipe_data: Recipe,
    image_list: Vec<String>,
) -> impl IntoView {
    let recipe_data = Arc::new(recipe_data);
    let img_url_prefix = format!(
        "/cdn/img/get/{}/",
        &recipe_data.name.to_lowercase(),
    );

    let ingredient_context = expect_context::<IngredientsContext>();

    let render_ingredients = |recipe_data: &Recipe, ingredient_context: &IngredientsContext| {
        view! {
            <ul>
                {recipe_data.ingredients
                    .iter()
                    .map(|i| Ok(view! {
                        <li>{
                            let ingredient_name = match ingredient_context.0.get(&i.ingredient_id) {
                                None => return Err(BoundaryError::new("Ingredient error".to_string())),
                                Some(val) => val,
                            };
                            format!(
                                "{}: {}",
                                ingredient_name.name,
                                i.ammount,
                            )
                        }</li>
                    }))
                    .collect_view()
                }
            </ul>
        }.into_view()
    };

    let icon_url = format!("{img_url_prefix}icon");

    view! {
        // Opengraph tags
        // This generates some browser wanrings but i want it that way
        { {
            view! {
                <Meta property="og:title" content=recipe_data.name.clone()/>
                <Meta property="og:type" content="website"/>
                <Meta property="og:image" content=icon_url.clone()/>
                <Meta property="og:url" content=format!("/r/{}", &recipe_data.name)/>
            }
        }}

        <h1>{ &recipe_data.name }</h1>
        <img
            src=&icon_url
            style="max-width:66%;"
            alt="Icon"
        />
        <h5> "Ingredients:" </h5>
        {if recipe_data.ingredients.is_empty() {
            ().into_view()
        } else {
            let recipe_data = Arc::clone(&recipe_data);
            view! {
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
                    {move || render_ingredients(&recipe_data, &ingredient_context)}
                </ErrorBoundary>
            }.into_view()
        }}
        <h5> "Instruction:" </h5>
        {
            use crate::md_parser::{deafult_options, parse};
            parse(&recipe_data.instructions, deafult_options())
        }
        <h5> "Gallery:" </h5>
        {
            image_list.iter()
                .map(|i| view! {<img src=format!("{img_url_prefix}{i}")/>})
                .collect_view()
        }
    }
}
