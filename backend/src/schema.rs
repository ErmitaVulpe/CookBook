// @generated automatically by Diesel CLI.

diesel::table! {
    ammounts (id) {
        id -> Integer,
        recipe -> Text,
        kind -> Text,
        ammount -> Float,
        unit -> Text,
    }
}

diesel::table! {
    ingredients (name) {
        name -> Text,
    }
}

diesel::table! {
    key_value (key) {
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    recipes (name) {
        name -> Text,
        owner -> Text,
        instructions -> Text,
    }
}

diesel::table! {
    users (username) {
        username -> Text,
        password_hash -> Text,
    }
}

diesel::joinable!(ammounts -> ingredients (kind));
diesel::joinable!(ammounts -> recipes (recipe));
diesel::joinable!(recipes -> users (owner));

diesel::allow_tables_to_appear_in_same_query!(
    ammounts,
    ingredients,
    key_value,
    recipes,
    users,
);
