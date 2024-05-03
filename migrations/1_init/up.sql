CREATE TABLE recipes (
    recipe_id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_name TEXT NOT NULL,
    instructions TEXT NOT NULL,
    next_photo_id INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE ingredients (
    ingredient_id INTEGER PRIMARY KEY AUTOINCREMENT,
    ingredient_name TEXT NOT NULL,
    is_indexable BOOL NOT NULL
);

CREATE TABLE recipe_ingredients (
    recipe_id INTEGER NOT NULL REFERENCES recipes(recipe_id),
    ingredient_id INTEGER NOT NULL REFERENCES ingredients(ingredient_id),
    ammount TEXT NOT NULL,
    PRIMARY KEY (recipe_id, ingredient_id)
);
