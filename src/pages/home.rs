use leptos::*;
use std::ops::Deref;

/// Renders the home page of your application.
#[component]
pub fn Home() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    let file_input = create_node_ref::<html::Input>();

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <br/>
        <input
            name="Upload photos"
            type="file"
            node_ref=file_input
            accept="image/*"
            multiple

            on:change=move |_| {
                let file_list: web_sys::FileList = file_input().unwrap().deref().files().unwrap();
                for i in 0..file_list.length() {
                    logging::log!("{:#?}", file_list.get(i).unwrap());
                }
                logging::log!("{:#?}", file_list);
            }
        />
    }
}
