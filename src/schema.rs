// @generated automatically by Diesel CLI.

diesel::table! {
    ingredients (id) {
        id -> Integer,
        name -> Text,
        is_indexable -> Bool,
    }
}

diesel::table! {
    recipe_ingredients (id) {
        id -> Integer,
        recipe_name -> Text,
        ingredient_id -> Integer,
        ammount -> Text,
    }
}

diesel::table! {
    recipes (name) {
        name -> Text,
        instructions -> Text,
    }
}

diesel::joinable!(recipe_ingredients -> ingredients (ingredient_id));
diesel::joinable!(recipe_ingredients -> recipes (recipe_name));

diesel::allow_tables_to_appear_in_same_query!(
    ingredients,
    recipe_ingredients,
    recipes,
);
