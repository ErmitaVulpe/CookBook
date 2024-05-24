pub mod create_recipe;
pub mod delete_recipe;

use leptos::*;
use leptos_router::*;
use rand::RngCore;
use std::collections::BTreeMap;

use crate::api::{
    self,
    auth::{LoggedStatus, UserRaw},
    recipes::Ingredient,
};

#[derive(Clone, Debug)]
pub struct IngredientsContext(BTreeMap<i32, Ingredient>);
#[derive(Clone, Debug)]
pub struct RecipeNamesContext(Vec<String>);

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
        <h1>{
            // Le funny
            let mut rng = rand::thread_rng();
            if (rng.next_u32() % 100) == 0 {
                "Adnim palen"
            } else {
                "Admin panel"
            }
        }</h1>
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
                        let recipe_names_req = api::recipes::get_recipe_names();

                        let ingredients = match ingredients_req.await {
                            Ok(val) => Some(val),
                            Err(err) => {
                                loading_errors.push(
                                    format!("Error loading ingredients: {}", err)
                                );
                                None
                            },
                        };

                        let recipe_names = match recipe_names_req.await {
                            Ok(val) => Some(val),
                            Err(err) => {
                                loading_errors.push(
                                    format!("Error loading recipe list: {}", err)
                                );
                                None
                            },
                        };

                        if loading_errors.is_empty() {
                            // Safe since already checked
                            unsafe { Ok((
                                ingredients.unwrap_unchecked(),
                                recipe_names.unwrap_unchecked(),
                        ))}
                        } else {
                            Err(loading_errors)
                        }
                    }
                );

                view! {
                    <p> "Logged in" </p>
                    <button on:click=move |_| log_out.dispatch(())>"Log out"</button>
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
                                Ok((ingredients, recipe_names)) => {
                                    let ingredients = {
                                        let mut map = BTreeMap::new();
                                        for ingredient in ingredients {
                                            map.insert(ingredient.id, ingredient);
                                        }
                                        map
                                    };

                                    provide_context(create_rw_signal(IngredientsContext(ingredients)));
                                    provide_context(create_rw_signal(RecipeNamesContext(recipe_names)));

                                    view! {
                                        <Outlet />
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
                id="username"
                type="text"
                required
                placeholder="Username"
                autocomplete="username"
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
                id="current-password"
                type="password"
                required
                placeholder="Password"
                autocomplete="current-password"
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
pub fn ToolList() -> impl IntoView {
    view! {
        <ul>
            <li><A href="create_recipe"> "Create a recipe" </A></li>
            <li><A href="delete_recipe"> "Delete recipes" </A></li>
            // <li><A href=""> "" </A></li>
        </ul>
    }
}


#[component]
pub fn GoBack() -> impl IntoView {
    view! {
        <div style="padding: 1rem 0;">
            <A href="/admin"> "<- Go back" </A>
        </div>
    }
}
