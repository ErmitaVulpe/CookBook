use crate::{auth, db, schema};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;


#[derive(Debug, Clone, Queryable, AsChangeset, Insertable, Serialize, Deserialize)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Queryable, AsChangeset, Insertable, Serialize, Deserialize)]
#[diesel(table_name = schema::ingredients)]
pub struct Ingredient {
    pub name: String,
}

#[derive(Debug, Clone, Queryable, AsChangeset, Insertable, Serialize, Deserialize)]
#[diesel(table_name = schema::ammounts)]
pub struct AmmountInsertable {
    pub recipe: String,
    pub kind: String,
    pub ammount: f32,
    pub unit: String,
}


#[derive(Debug, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name = schema::key_value)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct AppData {
    pub pool: db::Pool,
    pub jwt_conf: auth::jwt::JwtConfig,
}
