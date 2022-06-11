use crate::{
    config::auth::AuthManager,
    config::DbPool,
    errors::MyError,
    models::users::User,
    selectors::users::{get_user_by_credential, LogInInput},
    services::users::{add_user, SignUpInput},
};

use actix_web::{get, post, web};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(signup).service(login).service(validate_token);
}

#[post("users/signup")]
async fn signup(
    db: web::Data<DbPool>,
    input_user: web::Json<SignUpInput>,
) -> actix_web::Result<web::Json<User>, MyError> {
    match db.get() {
        Err(_) => Err(MyError::InternalServerError),
        Ok(conn) => match add_user(conn, input_user.into_inner()) {
            Ok(user) => Ok(web::Json(user)),
            Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                Err(MyError::UserAlreadyExists)
            }
            Err(err) => Err(MyError::DieselError(err)),
        },
    }
}

#[derive(Serialize, Deserialize)]
struct Token {
    token: String,
}

fn wrap_token_maybe_as_response(
    token_maybe: Option<String>,
) -> actix_web::Result<web::Json<Token>, MyError> {
    match token_maybe {
        Some(token) => Ok(web::Json(Token { token: token })),
        None => Err(MyError::TokenCreationError),
    }
}

#[get("users/login")]
async fn login(
    db: web::Data<DbPool>,
    input_user: web::Json<LogInInput>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<web::Json<Token>, MyError> {
    match db.get() {
        Err(_) => Err(MyError::InternalServerError),
        Ok(conn) => match get_user_by_credential(conn, input_user.into_inner()) {
            Ok(user) => wrap_token_maybe_as_response(auth_mgr.create_token(user.id)),
            Err(_) => Err(MyError::UserDoesNotExists),
        },
    }
}

#[get("users/validate_token")]
async fn validate_token(
    web::Json(Token { token }): web::Json<Token>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<String, MyError> {
    if auth_mgr.validate_token::<i32>(token.clone()) {
        Ok("true".to_string())
    } else {
        Err(MyError::TokenValidationError)
    }
}
