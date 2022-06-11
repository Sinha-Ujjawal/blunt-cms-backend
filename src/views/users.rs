use crate::{
    config::auth::AuthManager, config::DbPool, errors::MyError,
    selectors::users::get_user_by_username, services::users::add_user, utils::validate_password,
};

use actix_web::{get, post, web};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(signup).service(login).service(validate_token);
}

#[derive(Debug, Serialize, Deserialize)]
struct SignUpInput {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct UserData {
    id: i32,
    username: String,
}

#[post("users/signup")]
async fn signup(
    db: web::Data<DbPool>,
    input_user: web::Json<SignUpInput>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    match db.get() {
        Err(_) => Err(MyError::InternalServerError),
        Ok(conn) => match add_user(
            conn,
            input_user.username.clone(),
            input_user.password.clone(),
        ) {
            Ok(user) => Ok(web::Json(UserData {
                id: user.id,
                username: user.username,
            })),
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

#[derive(Debug, Serialize, Deserialize)]
struct LogInInput {
    username: String,
    password: String,
}

#[get("users/login")]
async fn login(
    db: web::Data<DbPool>,
    input_user: web::Json<LogInInput>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<web::Json<Token>, MyError> {
    match db.get() {
        Err(_) => Err(MyError::InternalServerError),
        Ok(conn) => match get_user_by_username(conn, input_user.username.clone()) {
            Ok(user) => {
                if validate_password(input_user.password.clone(), user.password_hash) {
                    wrap_token_maybe_as_response(auth_mgr.create_token(user.id))
                } else {
                    Err(MyError::IncorrectPassword)
                }
            }
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
