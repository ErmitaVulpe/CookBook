#[cfg(feature = "ssr")]
use diesel::{prelude::*, insert_into};

use leptos::*;
use leptos::server_fn::codec;
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::Ingredient;
use super::Error;

#[cfg(feature = "ssr")]
use super::auth::{check_if_logged, LoggedStatus};

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

    let app_data = super::extract_app_data().await?;
    let mut conn = app_data.get_conn()?;

    let result = ingredients
        .select(Ingredient::as_select())
        .load(&mut conn);

    result.map_err(|e| {
        ServerFnError::new(e.to_string())
    })
}

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn create_recipe(recipe: Recipe) -> Result<Result<(), Error>, ServerFnError> {
    let app_data = leptos_actix::extract::<actix_web::web::Data<crate::AppData>>()
        .await.map(|i| i.into_inner())?;
    let request = expect_context::<actix_web::HttpRequest>();

    Ok(match check_if_logged(&app_data.jwt, &request) {
        LoggedStatus::LoggedIn => {
            let mut conn = app_data.get_conn()?;

            let result = conn.transaction(|conn| {
                {

                    use crate::schema::recipes::dsl::*;
                    insert_into(recipes).values((
                        name.eq(recipe.name),
                        instructions.eq(recipe.instructions),
                    )).execute(conn)
                    .map(|_| ())?
                }
                {
                    use crate::schema::recipe_ingredients::dsl::*;
                    let ingredints = recipe.ingredients;
                    let other_recipe_name = &recipe.name;
                    // TODO Destruct recipe to satisfy borrow checker

                    let asd = ingredints.iter().map(|i| {
                        i.to_insertable(other_recipe_name)
                    }).collect::<Vec<_>>();

                    insert_into(recipe_ingredients).values(
                        &asd
                    ).execute(conn)
                    .map(|_| ())
                }
            });

            Ok(())
        },
        LoggedStatus::LoggedOut => {
            Err(Error::Unauthorized)
        },
    })
}

