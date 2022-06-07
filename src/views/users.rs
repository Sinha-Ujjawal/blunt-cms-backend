use crate::errors::MyError;
use crate::models::users::User;
use crate::services::users::{add_user, InputUser};
use crate::Pool;
use actix_web::{post, web};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};

#[post("/user/signup")]
pub async fn signup(
    db: web::Data<Pool>,
    input_user: web::Json<InputUser>,
) -> actix_web::Result<web::Json<User>, MyError> {
    match add_user(db, input_user) {
        Ok(user) => Ok(web::Json(user)),
        Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Err(MyError::UserAlreadyExists)
        }
        Err(err) => Err(MyError::DieselError(err)),
    }
}
