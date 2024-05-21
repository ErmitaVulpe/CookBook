#[cfg(feature = "ssr")]
use diesel::{prelude::*, insert_into};

use leptos::*;
use leptos::server_fn::codec;
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::{Ingredient, Recipe as DbRecipe, RecipeIngredients};
use super::Error;

#[cfg(feature = "ssr")]
use super::{
    auth::{check_if_logged, LoggedStatus}, 
    extract_app_data
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Recipe {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "ins")]
    pub instructions: String,
    #[serde(rename = "ing")]
    pub ingredients: Vec<IngredientWithAmount>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IngredientWithAmount {
    #[serde(rename = "id")]
    pub ingredient_id: i32,
    #[serde(rename = "a")]
    pub ammount: String,
}

#[cfg(feature = "ssr")]
impl IngredientWithAmount {
    pub fn to_insertable(&self, recipe_name: &str) -> OwnedIngredientWithAmount {
        OwnedIngredientWithAmount {
            recipe_name: recipe_name.to_string(),
            ingredient_id: self.ingredient_id,
            ammount: self.ammount.clone(),
        }
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = crate::schema::recipe_ingredients)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct OwnedIngredientWithAmount {
    recipe_name: String,
    ingredient_id: i32,
    ammount: String,
}



#[server(input = codec::GetUrl, output = codec::Cbor)]
pub async fn get_ingredients() -> Result<Vec<Ingredient>, ServerFnError> {
    use crate::schema::ingredients::dsl::*;

    let app_data = extract_app_data().await?;
    let mut conn = app_data.get_conn()?;

    let result = ingredients
        .select(Ingredient::as_select())
        .load(&mut conn);

    result.map_err(|e| {
        ServerFnError::new(e.to_string())
    })
}

#[server(input = codec::GetUrl, output = codec::Cbor)]
pub async fn get_recipe_ingredients(name: String) -> Result<Vec<RecipeIngredients>, ServerFnError> {
    use crate::schema::recipe_ingredients::dsl::*;

    let app_data = extract_app_data().await?;
    let mut conn = app_data.get_conn()?;

    let result = recipe_ingredients
        .filter(recipe_name.eq(name))
        .select(RecipeIngredients::as_select())
        .load(&mut conn);

    result.map_err(|e| {
        ServerFnError::new(e.to_string())
    })
}

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn create_recipe(recipe: Recipe) -> Result<Result<(), Error>, ServerFnError> {
    let app_data = extract_app_data().await?;
    let request = expect_context::<actix_web::HttpRequest>();

    match check_if_logged(&app_data.jwt, &request) {
        LoggedStatus::LoggedIn => {
            let mut conn = app_data.get_conn()?;

            let Recipe {
                name: new_recipe_name,
                instructions: new_recipe_instructions,
                ingredients: new_recipe_ingredients,
            } = recipe;

            let result = conn.transaction(move |conn| {
                {
                    use crate::schema::recipes::dsl::*;
                    insert_into(recipes).values((
                        name.eq(&new_recipe_name),
                        instructions.eq(&new_recipe_instructions),
                    )).execute(conn)
                    .map(|_| ())?
                }
                {
                    use crate::schema::recipe_ingredients::dsl::*;

                    let asd = new_recipe_ingredients.iter().map(|i| {
                        i.to_insertable(&new_recipe_name)
                    }).collect::<Vec<_>>();

                    insert_into(recipe_ingredients).values(
                        &asd
                    ).execute(conn)
                    .map(|_| ())
                }
            });

            match result {
                Ok(_) => Ok(Ok(())),
                Err(err) => Err(ServerFnError::new(err.to_string())),
            }
        },
        LoggedStatus::LoggedOut => {
            Ok(Err(Error::Unauthorized))
        },
    }
}

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn delete_recipes(recipe_names: Vec<String>) -> Result<Result<(), Error>, ServerFnError> {
    let app_data = extract_app_data().await?;
    let request = expect_context::<actix_web::HttpRequest>();

    match check_if_logged(&app_data.jwt, &request) {
        LoggedStatus::LoggedIn => {
            let mut conn = app_data.get_conn()?;

            let result = conn.transaction(move |conn| {
                use crate::schema::recipes::dsl::*;

                diesel::delete(recipes.filter(name.eq_any(&recipe_names)))
                    .execute(conn)
                    .map(|_| ())
            });

            match result {
                Ok(_) => Ok(Ok(())),
                Err(err) => Err(ServerFnError::new(err.to_string())),
            }
        },
        LoggedStatus::LoggedOut => {
            Ok(Err(Error::Unauthorized))
        },
    }
}

#[server(input = codec::GetUrl, output = codec::Cbor)]
pub async fn get_recipe_names() -> Result<Vec<String>, ServerFnError> {
    let app_data = extract_app_data().await?;
    let mut conn = app_data.get_conn()?;

    use crate::schema::recipes::dsl::*;
    let result = recipes
        .select(name)
        .load::<String>(&mut conn);

    result.map_err(|e| ServerFnError::new(e.to_string()))
}

