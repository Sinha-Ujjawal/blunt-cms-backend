use crate::{
    config::DbPoolConnection,
    models::users::{NewUser, User},
    schema::users::dsl::*,
    utils::hash_password,
};

use diesel::expression::dsl::now;
use diesel::prelude::*;

pub fn add_user<'a>(
    conn: &DbPoolConnection,
    user_name: &'a str,
    password: &'a str,
) -> Result<User, diesel::result::Error> {
    let new_user = NewUser {
        username: user_name,
        password_hash: &hash_password(password.as_bytes()),
    };
    let res = diesel::insert_into(users)
        .values(&new_user)
        .get_result(conn)?;
    Ok(res)
}

pub fn update_user_password<'a>(
    conn: &DbPoolConnection,
    user_id: i32,
    new_password: &'a str,
) -> Result<User, diesel::result::Error> {
    diesel::update(users.filter(id.eq(user_id)))
        .set((
            password_hash.eq(&hash_password(new_password.as_bytes())),
            updated_at.eq(now),
        ))
        .get_result(conn)
}
