use crate::config::auth::AuthManager;
use crate::errors::MyError;
use crate::models::users::User;
use crate::selectors::users::{get_user_by_credential, LogInInput};
use crate::services::users::{add_user, SignUpInput};
use crate::Pool;
use actix_web::{get, post, web};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use serde::Serialize;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(signup).service(login);
}

#[post("users/signup")]
async fn signup(
    db: web::Data<Pool>,
    input_user: web::Json<SignUpInput>,
) -> actix_web::Result<web::Json<User>, MyError> {
    match add_user(db, input_user) {
        Ok(user) => Ok(web::Json(user)),
        Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Err(MyError::UserAlreadyExists)
        }
        Err(err) => Err(MyError::DieselError(err)),
    }
}

#[derive(Serialize)]
struct Token {
    token: String,
}

#[get("users/login")]
async fn login(
    db: web::Data<Pool>,
    input_user: web::Json<LogInInput>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<web::Json<Token>, MyError> {
    match get_user_by_credential(db, input_user) {
        Ok(user) => match auth_mgr.create_token(user) {
            Some(token) => Ok(web::Json(Token { token: token })),
            None => Err(MyError::TokenCreationError),
        },
        Err(_) => Err(MyError::UserDoesNotExists),
    }
}
