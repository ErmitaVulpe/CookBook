// @generated automatically by Diesel CLI.

diesel::table! {
    ingredients (id) {
        id -> Integer,
        name -> Text,
        is_indexable -> Bool,
    }
}

diesel::table! {
    recipe_ingredients (recipe_id, ingredient_id) {
        recipe_id -> Integer,
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

diesel::allow_tables_to_appear_in_same_query!(
    ingredients,
    recipe_ingredients,
    recipes,
);
