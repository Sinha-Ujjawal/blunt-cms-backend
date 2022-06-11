use crate::{
    config::DbPoolConnection,
    models::users::{NewUser, User},
    schema::users::dsl::*,
    utils::hash_password,
};

use diesel::prelude::*;

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

pub fn update_user_password(
    conn: DbPoolConnection,
    user_id: i32,
    new_password: String,
) -> Result<User, diesel::result::Error> {
    diesel::update(users.filter(id.eq(user_id)))
        .set(password_hash.eq(&hash_password(new_password)))
        .get_result(&conn)
}
