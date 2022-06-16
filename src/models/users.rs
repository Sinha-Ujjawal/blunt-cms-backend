use crate::schema::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Identifiable, Serialize, Deserialize, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub is_admin: bool,
}
