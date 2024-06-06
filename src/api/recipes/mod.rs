#[cfg(feature = "ssr")]
use diesel::{prelude::*, insert_into};

use leptos::*;
use leptos::server_fn::codec;
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::{
    Ingredient,
    Recipe as DbRecipe,
    RecipeIngredients,
    IngredientWithAmount
};
use super::Error;

#[cfg(feature = "ssr")]
use super::{
    auth::{check_if_logged, LoggedStatus}, 
    extract_app_data
};

pub const MAX_RECIPE_NAME_LENGHT: usize = 100;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Recipe {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "ins")]
    pub instructions: String,
    #[serde(rename = "ing")]
    pub ingredients: Vec<IngredientWithAmount>,
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
pub async fn get_recipe(recipe_name: String) -> Result<Option<Recipe>, ServerFnError> {
    let app_data = extract_app_data().await?;
    let mut conn = app_data.get_conn()?;

    let recipe = {
        use crate::schema::recipes::dsl::*;

        let result = recipes
            .find(&recipe_name)
            .select(models::Recipe::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        match result {
            Some(val) => val,
            None => return Ok(None),
        }
    };

    let ingredients = {
        use crate::schema::recipe_ingredients::dsl;

        dsl::recipe_ingredients
            .filter(dsl::recipe_name.eq(&recipe_name))
            .select(IngredientWithAmount::as_select())
            .load(&mut conn)
            .map_err(|e| ServerFnError::new(e.to_string()))?
    };

    Ok(Some(Recipe {
        name: recipe_name,
        instructions: recipe.instructions,
        ingredients,
    }))
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
        .filter(recipe_name.eq(name.to_lowercase()))
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

            let result = conn.transaction(|conn| {
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

            use diesel::result::{DatabaseErrorKind::UniqueViolation, Error::DatabaseError};
            match result {
                Err(DatabaseError(UniqueViolation, _)) => 
                    return Err(ServerFnError::new("Recipe with this name already exists".to_string())),
                Err(err) => return Err(ServerFnError::new(err.to_string())),
                _ => {},
            }

            if let Err(err) = result {
                return Err(ServerFnError::new(err.to_string()));
            }

            let result = app_data.cdn.transaction(|c| {
                c.create_recipe(&new_recipe_name)
            });

            match result {
                Ok(()) => Ok(Ok(())),
                Err(err) => Err(err.into()),
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
            let recipe_names = recipe_names
                .iter()
                .map(|s| s.to_lowercase())
                .collect::<Vec<_>>();

            let result = conn.transaction(|conn| {
                use crate::schema::recipes::dsl::*;

                diesel::delete(recipes.filter(name.eq_any(&recipe_names)))
                    .execute(conn)
                    .map(|_| ())
            });

            if let Err(err) = result {
                return Err(ServerFnError::new(err.to_string()));
            }

            let result = app_data.cdn.transaction(|c| {
                for recipe in recipe_names {
                    c.delete_recipe(&recipe)?;
                }
                Ok(())
            });

            match result {
                Ok(()) => Ok(Ok(())),
                Err(err) => Err(err.into()),
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

#[server(input = codec::GetUrl, output = codec::Cbor)]
pub async fn get_images_for_recipe(recipe_name: String) -> Result<Vec<String>, ServerFnError> {
    let app_data = extract_app_data().await?;
    let images = app_data.cdn.get_image_list(&recipe_name)?;
    Ok(images)
}


// Maybe some day codec::MultipartFormData will be supported with actix

// /// fields:
// /// - r - Recipe name
// /// - d - Image data
// #[server(input = codec::MultipartFormData, output = codec::Cbor)]
// pub async fn upload_icon(data: codec::MultipartData) -> Result<Result<(), Error>, ServerFnError> {
//     let mut data = data.into_inner().unwrap();

//     // let app_data = extract_app_data().await?;

//     while let Ok(Some(mut field)) = data.next_field().await {
//         let name =
//             field.file_name().expect("no filename on field").to_string();
//         while let Ok(Some(chunk)) = field.chunk().await {
//             let len = chunk.len();
//             println!("[{name}]\t{len}");
//             // in a real server function, you'd do something like saving the file here
//         }
//     }

//     Ok(Ok(()))
// }