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
    recipe_id INTEGER NOT NULL REFERENCES recipes(recipe_id),
    ingredient_id INTEGER NOT NULL REFERENCES ingredients(ingredient_id),
    ammount TEXT NOT NULL,
    PRIMARY KEY (recipe_id, ingredient_id)
);
