use super::super::models::users::User;
use super::super::services::users::{add_user, InputUser};
use super::super::Pool;
use actix_web::{post, web};

#[post("/user/signup")]
pub async fn signup(
    db: web::Data<Pool>,
    input_user: web::Json<InputUser>,
) -> actix_web::Result<web::Json<User>> {
    let user = add_user(db, input_user).unwrap();
    Ok(web::Json(user))
}
