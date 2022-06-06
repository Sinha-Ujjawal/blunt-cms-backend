use super::super::models::users::{NewUser, User};
use super::super::Pool;
use crate::diesel::RunQueryDsl;
use crate::schema::users::dsl::*;
use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputUser {
    pub username: String,
    pub password: String,
}

pub fn add_user(
    db: web::Data<Pool>,
    input_user: web::Json<InputUser>,
) -> Result<User, diesel::result::Error> {
    let conn = db.get().unwrap();
    let new_user = NewUser {
        username: &input_user.username,
        password_hash: &input_user.password,
    };
    let res = diesel::insert_into(users)
        .values(&new_user)
        .get_result(&conn)?;
    Ok(res)
}
