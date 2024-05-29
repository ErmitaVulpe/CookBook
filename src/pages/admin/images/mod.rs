use leptos::*;
use super::{GoBack, RecipeNamesContext};

pub mod delete_image;
pub mod upload_image;

#[component]
fn SelectRecipe(
    #[prop(into)]
    recipe_list: Signal<RecipeNamesContext>,
    selected_recipe: RwSignal<Option<String>>,
    #[prop(default = false.into())]
    #[prop(into)]
    disabled: MaybeSignal<bool>,
) -> impl IntoView {


    view! {
        <select 
            prop:disabled=move || disabled.get()
            name="select_recipe"
            on:change=move |ev| {
                let value = event_target_value(&ev);
                selected_recipe.set({
                    match value {
                        x if x.starts_with('0') => None,
                        x if x.starts_with('1') => Some(x[1..].to_string()),
                        _ => unreachable!("Options have to have value starting with 0 or 1"),
                }
            });
        }>
            <option
                value='0'
                selected=move || selected_recipe.get().is_none()
            > "-" </option>
            <For
                each=move || recipe_list.get().0
                key=|ingredient| ingredient.clone()
                children=move |ingredient|{
                    let value = format!("1{ingredient}");
                    view! {
                        <option
                            value=value.clone()
                            // .as_ref() avoids an additional allocation
                            selected=move || selected_recipe.get().as_ref() == Some(&value)
                        >
                            {ingredient}
                        </option>
                    }
                }
            />
        </select>
    }
}
