use crate::{config::Pool, models::users::User, schema::users::dsl::*};

use actix_web::web;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogInInput {
    pub username: String,
    pub password: String,
}

pub fn get_user_by_credential(
    db: web::Data<Pool>,
    input_user: web::Json<LogInInput>,
) -> Result<User, diesel::result::Error> {
    let conn = db.get().unwrap();

    let res = users
        .filter(
            username
                .eq(&input_user.username)
                .and(password_hash.eq(&input_user.password)),
        )
        .first::<User>(&conn)?;

    Ok(res)
}
