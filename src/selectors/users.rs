use crate::{config::DbPoolConnection, models::users::User, schema::users::dsl::*};

use diesel::prelude::*;

pub fn get_user_by_username(
    conn: DbPoolConnection,
    user_name: String,
) -> Result<User, diesel::result::Error> {
    users.filter(username.eq(&user_name)).first::<User>(&conn)
}
