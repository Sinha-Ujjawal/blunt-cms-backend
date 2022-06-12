use crate::{
    config::auth::AuthManager,
    config::{DbPool, DbPoolConnection},
    errors::MyError,
    models::users::User,
    selectors, services, utils,
};

use actix_web::{get, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(signup)
        .service(login)
        .service(validate_token)
        .service(change_password);
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

#[derive(Debug, Serialize, Deserialize)]
struct SignUpInput {
    username: String,
    password: String,
}

async fn add_user(
    conn: DbPoolConnection,
    input_user: SignUpInput,
) -> actix_web::Result<User, MyError> {
    web::block(move || {
        services::users::add_user(
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
    })
}

#[post("users/signup")]
async fn signup(
    db: web::Data<DbPool>,
    input_user: web::Json<SignUpInput>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user = add_user(conn, input_user.into_inner()).await?;
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

async fn get_user_by_username(
    conn: DbPoolConnection,
    user_name: String,
) -> actix_web::Result<User, MyError> {
    web::block(move || selectors::users::get_user_by_username(conn, user_name))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|_| MyError::UserDoesNotExists)
}

#[get("users/login")]
async fn login(
    db: web::Data<DbPool>,
    input_user: web::Json<LogInInput>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<web::Json<Token>, MyError> {
    let password = input_user.password.clone();
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user = get_user_by_username(conn, input_user.username.clone()).await?;
    if utils::validate_password(password.clone(), user.password_hash) {
        let token = web::block(move || auth_mgr.create_token(user.id))
            .await
            .map_err(|_| MyError::InternalServerError)?
            .ok_or(MyError::TokenCreationError)?;
        Ok(web::Json(Token { token: token }))
    } else {
        Err(MyError::IncorrectPassword)
    }
}

#[derive(Serialize, Deserialize)]
struct AuthedUser {
    user_id: i32,
    user: User,
}

impl AuthedUser {
    fn from_token<'a>(
        conn: DbPoolConnection,
        auth_mgr: web::Data<AuthManager>,
        token: &'a str,
    ) -> Option<Self> {
        let user_id = auth_mgr.extract_claim::<i32>(token).ok()?;
        let user = selectors::users::get_user_by_user_id(conn, user_id).ok()?;
        Some(AuthedUser {
            user_id: user_id,
            user: user,
        })
    }
}

async fn get_authed_user_from_bearer_token(
    conn: DbPoolConnection,
    auth_mgr: web::Data<AuthManager>,
    bearer_token: String,
) -> actix_web::Result<AuthedUser, MyError> {
    web::block(move || AuthedUser::from_token(conn, auth_mgr, &bearer_token))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .ok_or(MyError::TokenValidationError)
}

#[get("users/validate_token")]
async fn validate_token(
    bearer_auth: BearerAuth,
    auth_mgr: web::Data<AuthManager>,
    db: web::Data<DbPool>,
) -> actix_web::Result<String, MyError> {
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;

    let _: AuthedUser =
        get_authed_user_from_bearer_token(conn, auth_mgr, bearer_auth.token().to_string()).await?;

    Ok("true".to_string())
}

#[derive(Serialize, Deserialize)]
struct UserChangePasswordInput {
    new_password: String,
}

async fn update_user_password(
    conn: DbPoolConnection,
    user_id: i32,
    new_password: String,
) -> actix_web::Result<User, MyError> {
    web::block(move || services::users::update_user_password(conn, user_id, new_password))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[post("users/change_password")]
async fn change_password(
    web::Json(UserChangePasswordInput { new_password }): web::Json<UserChangePasswordInput>,
    bearer_auth: BearerAuth,
    auth_mgr: web::Data<AuthManager>,
    db: web::Data<DbPool>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    let mut conn = db.get().map_err(|_| MyError::InternalServerError)?;

    let authed_user: AuthedUser =
        get_authed_user_from_bearer_token(conn, auth_mgr, bearer_auth.token().to_string()).await?;

    conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user = update_user_password(conn, authed_user.user_id, new_password).await?;

    Ok(web::Json(UserData::from_user(user)))
}
