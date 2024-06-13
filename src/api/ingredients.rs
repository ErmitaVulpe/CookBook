use leptos::*;
use leptos::server_fn::codec;
use serde::{Serialize, Deserialize};

#[cfg(feature = "ssr")]
use diesel::{
    prelude::*,
    delete,
    insert_into,
    result::{
        DatabaseErrorKind::UniqueViolation,
        Error::DatabaseError,
    },
};

use super::Error;
#[cfg(feature = "ssr")]
use super::{
    auth::{check_if_logged, LoggedStatus}, 
    extract_app_data
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CreateIngredientResult {
    Ok,
    IngredientExists,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeleteIngredientResult {
    Ok,
    /// Inner is list of recipies that use this ingredient
    IngredientInUse(Vec<String>),
}

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn create_ingredient(ingredeint_name: String, is_indexable: bool) -> Result<Result<CreateIngredientResult, Error>, ServerFnError> {
    let app_data = extract_app_data().await?;
    let request = expect_context::<actix_web::HttpRequest>();

    match check_if_logged(&app_data.jwt, &request) {
        LoggedStatus::LoggedOut => {
            Ok(Err(Error::Unauthorized))
        },
        LoggedStatus::LoggedIn => {
            use crate::schema::ingredients::dsl;

            let mut conn = app_data.get_conn()?;
            let result = insert_into(dsl::ingredients)
                .values((
                    dsl::name.eq(ingredeint_name),
                    dsl::is_indexable.eq(is_indexable),
                )).execute(&mut conn);

            match result {
                Ok(_) => Ok(Ok(CreateIngredientResult::Ok)),
                Err(DatabaseError(UniqueViolation, _)) => Ok(Ok(CreateIngredientResult::IngredientExists)),
                Err(err) => Err(ServerFnError::from(err)),
            }
        },
    }
}

#[server(input = codec::Cbor, output = codec::Cbor)]
pub async fn delete_ingredients(ingredeint_names: Vec<String>) -> Result<Result<Vec<DeleteIngredientResult>, Error>, ServerFnError> {
    let app_data = extract_app_data().await?;
    let request = expect_context::<actix_web::HttpRequest>();

    match check_if_logged(&app_data.jwt, &request) {
        LoggedStatus::LoggedOut => {
            Ok(Err(Error::Unauthorized))
        },
        LoggedStatus::LoggedIn => {
            use crate::schema::ingredients::dsl;

            let mut results = Vec::with_capacity(ingredeint_names.len());

            let mut conn = app_data.get_conn()?;
            for ingredient in ingredeint_names {
                let result = delete(
                    dsl::ingredients
                        .filter(dsl::name.eq(&ingredient))
                ).execute(&mut conn);

                println!("{:#?}", result);
                // TODO
            }
            

            Ok(Ok(results))
        },
    }
}
