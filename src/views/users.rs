use crate::{
    config::auth::AuthManager,
    config::DbPool,
    errors::MyError,
    models::users::User,
    selectors::users::{get_user_by_user_id, get_user_by_username},
    services::users::{add_user, update_user_password},
    utils::validate_password,
};

use actix_web::{get, post, web};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(signup)
        .service(login)
        .service(validate_token)
        .service(change_password);
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

impl UserData {
    pub fn from_user(user: User) -> Self {
        UserData {
            id: user.id,
            username: user.username,
        }
    }
}

#[post("users/signup")]
async fn signup(
    db: web::Data<DbPool>,
    input_user: web::Json<SignUpInput>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;

    let user = web::block(move || {
        add_user(
            conn,
            input_user.username.clone(),
            input_user.password.clone(),
        )
    })
    .await
    .map_err(|_| MyError::InternalServerError)?
    .map_err(|err| match err {
        DatabaseError(DatabaseErrorKind::UniqueViolation, _) => MyError::UserAlreadyExists,
        _ => MyError::DieselError(err),
    })?;

    Ok(web::Json(UserData::from_user(user)))
}

#[derive(Serialize, Deserialize)]
struct Token {
    token: String,
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
    let password = input_user.password.clone();

    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user = web::block(move || get_user_by_username(conn, input_user.username.clone()))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|_| MyError::UserDoesNotExists)?;

    if validate_password(password.clone(), user.password_hash) {
        let token = web::block(move || auth_mgr.create_token(user.id))
            .await
            .map_err(|_| MyError::InternalServerError)?
            .ok_or(MyError::TokenCreationError)?;
        Ok(web::Json(Token { token: token }))
    } else {
        Err(MyError::IncorrectPassword)
    }
}

#[get("users/validate_token")]
async fn validate_token(
    web::Json(Token { token }): web::Json<Token>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<String, MyError> {
    let is_valid = web::block(move || auth_mgr.validate_token::<i32>(token.clone()))
        .await
        .map_err(|_| MyError::InternalServerError)?;

    if is_valid {
        Ok("true".to_string())
    } else {
        Err(MyError::TokenValidationError)
    }
}

#[derive(Serialize, Deserialize)]
struct UserChangePasswordInput {
    new_password: String,
    token: String,
}

#[post("users/change_password")]
async fn change_password(
    web::Json(UserChangePasswordInput {
        new_password,
        token,
    }): web::Json<UserChangePasswordInput>,
    auth_mgr: web::Data<AuthManager>,
    db: web::Data<DbPool>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    let user_id = web::block(move || auth_mgr.extract_claim::<i32>(token))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|_| MyError::TokenValidationError)?;

    let mut conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let _ = web::block(move || get_user_by_user_id(conn, user_id))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|_| MyError::UserDoesNotExists)?;

    conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user = web::block(move || update_user_password(conn, user_id, new_password))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))?;

    Ok(web::Json(UserData::from_user(user)))
}
