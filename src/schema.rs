// @generated automatically by Diesel CLI.

diesel::table! {
    ingredients (ingredient_id) {
        ingredient_id -> Nullable<Integer>,
        ingredient_name -> Text,
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
    recipes (recipe_id) {
        recipe_id -> Nullable<Integer>,
        recipe_name -> Text,
        instructions -> Text,
        next_photo_id -> Integer,
    }
}

diesel::joinable!(recipe_ingredients -> ingredients (ingredient_id));
diesel::joinable!(recipe_ingredients -> recipes (recipe_id));

diesel::allow_tables_to_appear_in_same_query!(
    ingredients,
    recipe_ingredients,
    recipes,
);
