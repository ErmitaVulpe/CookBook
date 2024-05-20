CREATE TABLE recipes (
    name TEXT PRIMARY KEY NOT NULL,
    instructions TEXT NOT NULL
);

CREATE TABLE ingredients (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    is_indexable BOOL NOT NULL
);

CREATE TABLE recipe_ingredients (
    recipe_name TEXT NOT NULL REFERENCES recipes(recipe_name),
    ingredient_id INTEGER NOT NULL REFERENCES ingredients(ingredient_id),
    ammount TEXT NOT NULL,
    PRIMARY KEY (recipe_name, ingredient_id)
);
