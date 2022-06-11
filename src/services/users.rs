use crate::{
    config::DbPoolConnection,
    models::users::{NewUser, User},
    schema::users::dsl::*,
    utils::hash_password,
};

use diesel::RunQueryDsl;

pub fn add_user(
    conn: DbPoolConnection,
    user_name: String,
    password: String,
) -> Result<User, diesel::result::Error> {
    let new_user = NewUser {
        username: &user_name,
        password_hash: &hash_password(password),
    };
    let res = diesel::insert_into(users)
        .values(&new_user)
        .get_result(&conn)?;
    Ok(res)
}
