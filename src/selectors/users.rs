use crate::{config::DbPoolConnection, models::users::User, schema::users::dsl::*};

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogInInput {
    pub username: String,
    pub password: String,
}

pub fn get_user_by_credential(
    conn: DbPoolConnection,
    input_user: LogInInput,
) -> Result<User, diesel::result::Error> {
    let res = users
        .filter(
            username
                .eq(&input_user.username)
                .and(password_hash.eq(&input_user.password)),
        )
        .first::<User>(&conn)?;

    Ok(res)
}
