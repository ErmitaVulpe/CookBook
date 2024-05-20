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
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    recipe_name TEXT NOT NULL REFERENCES recipes(name) ON DELETE CASCADE,
    ingredient_id INTEGER NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE,
    ammount TEXT NOT NULL,
    UNIQUE (recipe_name, ingredient_id)
);
