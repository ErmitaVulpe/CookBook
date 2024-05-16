use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use diesel::prelude::*;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::ingredients)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(id))]
pub struct Ingredient {
    pub id: i32,
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "ii")]
    pub is_indexable: bool,
}

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ingredient {
    pub id: i32,
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "ii")]
    pub is_indexable: bool,
}
