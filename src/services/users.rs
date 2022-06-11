use crate::{
    config::DbPoolConnection,
    models::users::{NewUser, User},
    schema::users::dsl::*,
};

use diesel::RunQueryDsl;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignUpInput {
    pub username: String,
    pub password: String,
}

pub fn add_user(
    conn: DbPoolConnection,
    input_user: SignUpInput,
) -> Result<User, diesel::result::Error> {
    let new_user = NewUser {
        username: &input_user.username,
        password_hash: &input_user.password,
    };
    let res = diesel::insert_into(users)
        .values(&new_user)
        .get_result(&conn)?;
    Ok(res)
}
