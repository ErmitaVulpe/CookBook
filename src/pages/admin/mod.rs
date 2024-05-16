use leptos::*;
use leptos_router::*;
use std::{
    collections::{BTreeSet, BTreeMap},
    ops::Deref,
};

use crate::api::{
    self,
    Error,
    auth::{LoggedStatus, UserRaw},
    recipes::{Recipe, Ingredient, IngredientWithAmount},
};

#[component]
pub fn Admin() -> impl IntoView {
    let is_logged = create_rw_signal(None::<bool>);
    let is_logged_memo = create_memo(move |_| is_logged.get());
    #[cfg(not(feature = "ssr"))]
    let check_if_logged = create_action(move |_: &()| async move {
        match api::auth::is_logged().await {
            Ok(val) => is_logged.update(|x| *x = Some(val == LoggedStatus::LoggedIn)),
            Err(err) => logging::error!("Error fetching data: {err}"),
        };
    });
    #[cfg(not(feature = "ssr"))]
    check_if_logged.dispatch(());

    let login_wait_for_response = create_rw_signal(false);
    let (login_error, set_login_error) = create_signal(None::<String>);
    let log_in = create_action(move |user: &UserRaw| {
        set_login_error.set(None);
        login_wait_for_response.set(true);
        let user = user.to_owned();
        async move {
            match api::auth::log_in(user.to_owned()).await {
                Ok(val) => {
                    set_login_error.update(|e| *e = match val {
                        LoggedStatus::LoggedIn => {
                            is_logged.update(|x| *x = Some(true));
                            None
                        },
                        LoggedStatus::LoggedOut => {
                            is_logged.update(|x| *x = Some(false));
                            Some("Invalid username or password".to_string())
                        },
                    });
                },
                Err(err) => set_login_error.update(|e| *e = Some(err.to_string())),
            };
            login_wait_for_response.set(false);
        }
    });

    let log_out = create_action(move |_: &()| 
        async move {
            match api::auth::log_out().await {
                Ok(()) => is_logged.update(|x| *x = Some(false)),
                Err(err) => logging::error!("Error fetching data: {err}"),
            }
        }
    );


    view! {
        <h1>"Admin panel"</h1>
        <A href="/"> "To home" </A>
        {move || match is_logged_memo.get() {
            None => view! { <p> "Loading" </p> }.into_view(),
            Some(false) => {
                view! { 
                    <LoginForm
                        action=log_in
                        error=login_error.into()
                        disabled=Signal::derive(move || login_wait_for_response.get())
                    />
                }
            }.into_view(),
            Some(true) => {
                let load_rescources = create_resource(
                    ||(),
                    |_| async move {
                        let mut loading_errors = Vec::<String>::new();

                        let ingredients_req = api::recipes::get_ingredients();

                        let ingredients = match ingredients_req.await {
                            Ok(val) => Some(val),
                            Err(err) => {
                                loading_errors.push(
                                    format!("Error loading ingredients: {}", err.to_string())
                                );
                                None
                            },
                        };

                        if loading_errors.len() == 0 {
                            // Safe since already checked
                            unsafe { Ok(
                                ingredients.unwrap_unchecked()
                            )}
                        } else {
                            Err(loading_errors)
                        }
                    }
                );

                view! {
                    <p> "Logged in" </p>
                    <button on:click=move |_| log_out.dispatch(())>"Log out"</button>
                    <button on:click=move |_| load_rescources.refetch()>"Reload rescources"</button>
                    <Suspense
                        fallback=move || view! { <p>"Loading..."</p> }
                    >
                        {move || {
                            load_rescources.get().map(|result| match result {
                                Err(err) => view! {
                                    <ul>{
                                        err.into_iter()
                                            .map(|n| view! { <li>{n}</li>})
                                            .collect_view()
                                    }</ul>
                                }.into_view(),
                                Ok(ingredients) => {
                                    let ingredients = create_rw_signal({
                                        let mut map = BTreeMap::new();
                                        for ingredient in ingredients {
                                            map.insert(ingredient.id, ingredient);
                                        }
                                        map
                                    });

                                    view! {
                                        <RecipeForm
                                            ingredients=ingredients
                                        />
                                    }
                                }.into_view(),
                            })
                        }}
                    </Suspense>
                }
            }.into_view(),
        }}
    }
}


#[component]
fn LoginForm(
    action: Action<UserRaw, ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());

    let dispatch_action =
        move || action.dispatch(UserRaw::new(&email.get(), &password.get()));

    let button_is_disabled = Signal::derive(move || {
        disabled.get() || password.get().is_empty() || email.get().is_empty()
    });

    view! {
        <form on:submit=|ev| ev.prevent_default()>
            <p> "Enter username and password:" </p>
            <p style="color:red; min-height:calc(10em/9);">
                {move || match error.get() {
                    Some(err) => err,
                    None => " ".to_string(),
                }}
            </p>
            <input
                type="text"
                required
                placeholder="Username"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: ev::KeyboardEvent| {
                    let val = event_target_value(&ev);
                    set_email.update(|v| *v = val);
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_email.update(|v| *v = val);
                }
            />
            <br/>
            <input
                type="password"
                required
                placeholder="Password"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: ev::KeyboardEvent| {
                    let val = event_target_value(&ev);
                    set_password.update(|p| *p = val);
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_password.update(|p| *p = val);
                }
            />
            <br/>
            <button
                prop:disabled=move || button_is_disabled.get()
                on:click=move |_| dispatch_action()
            >
                "Log in"
            </button>
        </form>
    }
}

#[component]
fn RecipeForm(
    #[prop(into)]
    ingredients: Signal<BTreeMap<i32, Ingredient>>,
) -> impl IntoView {
    let form_node = create_node_ref::<html::Form>();
    let name_node = create_node_ref::<html::Input>();
    let instructions_node = create_node_ref::<html::Textarea>();
    let icon_node = create_node_ref::<html::Input>();

    let selected_ingredients = create_rw_signal(Vec::new());

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
        <h2> "Create a new recipe" </h2>
        <form
            node_ref=form_node
            on:submit=move |ev| {
                ev.prevent_default();

                let new_recipe = Recipe {
                    name: name_node.get_untracked().unwrap().deref().value(),
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
    ingredients: Signal<BTreeMap<i32, Ingredient>>,
    selected_ingredients: RwSignal<Vec<IngredientWithAmount>>,
    #[prop(into)]
    disabled: Signal<bool>,
) -> impl IntoView {
    let select_ingredient = create_rw_signal(None::<i32>);
    let derived_ingredients = Signal::derive(move || {
        ingredients.get().values().cloned().collect::<Vec<_>>()
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
                            {ingredients.get().get(&ingredient.id).unwrap().name.to_string()}
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
                                    ingredients.get().get(&ingredient_with_ammount.ingredient_id).unwrap().name,
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

