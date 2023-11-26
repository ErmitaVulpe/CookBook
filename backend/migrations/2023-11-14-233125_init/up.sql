-- Your SQL goes here

CREATE TABLE users (
    username VARCHAR(31) PRIMARY KEY NOT NULL,
    password_hash VARCHAR(127) NOT NULL
);

CREATE TABLE recipes (
    name VARCHAR(255) PRIMARY KEY NOT NULL,
    owner VARCHAR(31) NOT NULL,
    instructions TEXT NOT NULL,

    FOREIGN KEY (owner) REFERENCES users(username) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE ingredients (
    name VARCHAR(255) PRIMARY KEY NOT NULL
);

CREATE TABLE ammounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    recipe VARCHAR(255) NOT NULL,
    kind VARCHAR(255) NOT NULL,
    ammount REAL NOT NULL,
    unit VARCHAR(31) NOT NULL,

    FOREIGN KEY (recipe) REFERENCES recipes(name),
    FOREIGN KEY (kind) REFERENCES ingredients(name)
);

CREATE TABLE key_value (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
