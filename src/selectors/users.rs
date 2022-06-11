use crate::{config::DbPoolConnection, models::users::User, schema::users::dsl::*};

use diesel::prelude::*;

pub fn get_user_by_username(
    conn: DbPoolConnection,
    user_name: String,
) -> Result<User, diesel::result::Error> {
    users.filter(username.eq(&user_name)).first::<User>(&conn)
}

pub fn get_user_by_user_id(
    conn: DbPoolConnection,
    user_id: i32,
) -> Result<User, diesel::result::Error> {
    users.filter(id.eq(&user_id)).first::<User>(&conn)
}
