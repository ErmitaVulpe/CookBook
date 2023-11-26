use crate::{auth, db::{self, Conn}, schema, models, unwrap_pretty::UnwrapPretty, validating};
use crate::macros::{exit_with_error, readln, readpw};
use std::io::{self, Write};
use rand::Rng;
use diesel::prelude::*;

// const HELP_INFO: &str = format!(r#"
// Usage: {name} [OPTIONS] COMMAND [ARGS]...

// Options:
//   -h, --help             Show this message and exit.
//   -v, --version          Show the version of the CLI.

// Commands:
//   command1               Description of command1.
//   command2               Description of command2.
//   command3               Description of command3.

// Additional Information:
//   - You can use '--help' with any command to get more details.
//   - For detailed information on a specific command, use:
//     your_cli COMMAND --help

// Examples:
//   - your_cli command1 --option1 value1
//   - your_cli command2 --option2 value2
// "#,
// name=env!("CARGO_PKG_NAME"),
// ).as_ref();

const SETUP_MENU: &str = r#"
Welcome to CookBook setup!

What would you like to do?
1) Create a new database file
2) Set new JWT secret
3) Create a new user
4) Recover password
5) Add a new ingredient
6) Remove an ingredient

> "#;

pub fn setup(db_path: &str) -> ! {
    // Print the menu
    print!("{}", SETUP_MENU);
    io::stdout().flush().unwrap();

    // Get user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    match input.trim() {
        "1" => { // Create new database file
            // Read in the path to the new database
            let mut db_path = readln!("Path to the new database file [./database.db]: ");
            if db_path.is_empty() { db_path = "./database.db".to_owned() }

            // Ask for the password for the admin
            let mut admin_pw = readpw!("Password for the admin account [admin]: ");
            // set a default password
            if admin_pw.is_empty() { admin_pw = "admin".to_owned() }
            else {
                let confirmation_pw = readpw!("Confirm password: ");
                if confirmation_pw != admin_pw {
                    exit_with_error!("Mismatched passwords")
                }
            }

            new_db_file(&db_path, &admin_pw);
            println!("New database file has been created and set up at: \"{}\". Remeber to update the \"DATABASE_PATH\" entry in your .env file if you have one", db_path);
        },
        "2" => { // Set new JWT secret
            let jwt_secret = readln!("New jwt secret (leave empty for random): ");
            new_jwt_secret(
                db_path,
                if jwt_secret.is_empty() { None } else { Some(jwt_secret) }
            );
        },
        "3" => { // Create new users
            let username = readln!("Username: ");
            if ! validating::is_valid_username(&username) {
                exit_with_error!("Invalid username. Check help page for more informations")}

            let pw = readpw!("Password: ");
            if ! validating::is_valid_password(&pw) {
                exit_with_error!("Invalid password. Check help page for more informations")}

            let confirmation_pw = readpw!("Confirm password: ");
            if confirmation_pw != pw {
                exit_with_error!("Mismatched passwords")
            }

            new_user(db_path, username.trim(), pw.trim());
        },
        "4" => { // Recover password
            let pool: db::Pool = db::establish_connection(format!("sqlite://{}", db_path));
            let mut conn: Conn = pool.get().unwrap();

            let username = readln!("Username: ");
            let result: Option<models::User> = schema::users::dsl::users
                .find(username.clone())
                .first(&mut conn)
                .optional()
                .unwrap_pretty("Error loading data");

            if result.is_none() { exit_with_error!("User not found"); }

            let pw = readpw!("Enter a new password: ");
            if ! validating::is_valid_password(&pw) {
                exit_with_error!("Invalid password. Check help page for more informations")}
            let confirmation_pw = readpw!("Confirm new password: ");
            if confirmation_pw != pw {
                exit_with_error!("Mismatched passwords")
            }

            let pw_hash = auth::hash_password(&pw);
            diesel::update(schema::users::dsl::users
                .filter(schema::users::dsl::username.eq(username)))
                .set(schema::users::dsl::password_hash.eq(pw_hash))
                .execute(&mut conn)
                .unwrap_pretty("Error setting password");

            println!("Successfully changed the password");
        },
        "5" => { // Add a new ingredient
            let name = readln!("Name of the new ingredient: ");
            if name.is_empty() { exit_with_error!("Ingredient name cannot be empty") }
            new_ingredient(db_path, &name);
        },
        "6" => { // Remove an ingredient
            println!("Before you proceed, keep in mind that removing an ingredient that is already being used in a recipe might have unexpected consequences!");
            let name = readln!("Name of the ingredient to remove: ");
            if name.is_empty() { exit_with_error!("Ingredient name cannot be empty") }
            remove_ingredient(db_path, &name);
        },
        _ => exit_with_error!("Invalid option")
    }

    std::process::exit(0);
}

pub fn validate_db(db_path: &str) -> db::Pool {
    // Validate database_path
    if std::fs::metadata(db_path).is_err() {
        exit_with_error!("Database file not found at specified path \"{}\", try creating it using the -s or --setup flag", db_path);
    }

    let pool: db::Pool = db::establish_connection(format!("sqlite://{}", db_path));
    let mut conn: Conn = pool.get().unwrap();

    use schema::users::dsl::*;

    let result = diesel::select(diesel::dsl::exists(users.filter(username.eq("admin"))))
        .get_result::<bool>(&mut conn);

    // Validate database
    match result {
        Err(_) | Ok(false) => exit_with_error!("Database seems empty or corrupted, try creating it using the -s or --setup flag"),
        _ => {}
    }

    pool
}

pub fn new_db_file(db_path: &str, admin_pw: &str) {
    if let Err(err) = std::fs::File::create(db_path) {
        exit_with_error!("Couldn't create file at \"{}\": {}", db_path, err);
    }

    if ! validating::is_valid_password(admin_pw) {
        exit_with_error!("Invalid password. Check help page for more informations")
    }

    let pool: db::Pool = db::establish_connection(format!("sqlite://{}", db_path));
    let mut conn: Conn = pool.get().unwrap();

    for query in db::SQL_DECLARATION.split(';') {
        let trimmed_query = query.trim();
        if !trimmed_query.is_empty() {
            if let Err(err) = diesel::sql_query(trimmed_query).execute(&mut conn) {
                exit_with_error!("Failed to execute the initial sql query: {}", err)
            }
        }
    }

    let result = diesel::insert_into(schema::users::dsl::users)
    .values(models::User {
        username: "admin".to_owned(),
        password_hash: auth::hash_password(admin_pw),
    })
    .execute(&mut conn);

    match result {
        Ok(_) => println!("A new database has been created"),
        Err(err) => exit_with_error!("Unexpected error during database creation: {}", err)
    }
}

pub fn new_jwt_secret(db_path: &str, jwt_secret: Option<String>) {
    let jwt_secret = match jwt_secret {
        Some(val) => val,
        None => {
            let mut rng = rand::thread_rng();
            (0..32)
                .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
                .collect()
        }
    };

    let pool = validate_db(db_path);
    let mut conn = pool.get().unwrap();
    db::key_value::set(&mut conn, "jwt_secret", &jwt_secret).unwrap_pretty(
        "Error setting the key value pair");

    println!("Successfully set new jwt secret");
}

pub fn new_user(db_path: &str, username: &str, password: &str) {
    if ! validating::is_valid_username(username) {
        exit_with_error!("Invalid username. Check help page for more informations")}
    if ! validating::is_valid_password(password) {
        exit_with_error!("Invalid password. Check help page for more informations")}

    let pool: db::Pool = validate_db(db_path);
    let mut conn: Conn = pool.get().unwrap();

    let result = diesel::insert_into(schema::users::dsl::users)
    .values(models::User {
        username: username.to_owned(),
        password_hash: auth::hash_password(password),
    })
    .execute(&mut conn);

    match result {
        Ok(_) => println!("A new user \"{}\" has been created", username),
        Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            exit_with_error!("The user \"{}\" already exists", username);
        }
        Err(err) => exit_with_error!("Unexpected error: {}", err)
    }
}

pub fn new_ingredient(db_path: &str, name: &str) {
    if ! validating::is_valid_ingredient_name(name) {
        exit_with_error!("Invalid ingredient name. Check help page for more informations")}

    let pool: db::Pool = validate_db(db_path);
    let mut conn: Conn = pool.get().unwrap();

    let result = diesel::insert_into(schema::ingredients::dsl::ingredients)
    .values(models::Ingredient {
        name: name.to_owned(),
    })
    .execute(&mut conn);

    match result {
        Ok(_) => println!("A new ingredient \"{}\" has been created", name),
        Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            exit_with_error!("The ingredient \"{}\" already exists", name);
        }
        Err(err) => exit_with_error!("Unexpected error: {}", err)
    }
}

pub fn remove_ingredient(db_path: &str, name: &str) {
    let pool: db::Pool = validate_db(db_path);
    let mut conn: Conn = pool.get().unwrap();

    let result = diesel::delete(
            schema::ingredients::dsl::ingredients
            .filter(schema::ingredients::dsl::name.eq(name))
        )
        .execute(&mut conn);

    match result {
        Ok(0) => exit_with_error!("No ingredient with this name found"),
        Ok(_) => println!("Successfuly removed the ingredient \"{}\"", name),
        Err(err) => exit_with_error!("Couldn't remove the ingredient: {}", err),
    }
}

pub fn set_socket(db_path: &str, socket: &str) {
    if ! validating::is_valid_socket(socket) {
        exit_with_error!("Invalid socket");
    }

    let pool = validate_db(db_path);
    let mut conn = pool.get().unwrap();
    db::key_value::set(&mut conn, "socket", socket).unwrap_pretty(
        "Error setting the key value pair");

    println!("Successfuly set the socket to \"{}\"", socket);
}

