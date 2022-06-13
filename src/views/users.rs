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
        .service(get_user)
        .service(validate_token)
        .service(change_password);
}

#[derive(Serialize, Deserialize)]
pub struct UserData {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
    pub is_super_admin: bool,
}

impl UserData {
    pub fn from_user_data(user_data: &selectors::admins::UserData) -> Self {
        use selectors::admins::UserData::*;
        match user_data {
            SimpleUser(user) => UserData {
                id: user.id,
                username: user.username.clone(),
                is_admin: false,
                is_super_admin: false,
            },
            AdminUser(user, is_super_admin) => UserData {
                id: user.id,
                username: user.username.clone(),
                is_admin: true,
                is_super_admin: *is_super_admin,
            },
        }
    }

    pub async fn from_bearer_token(
        db: web::Data<DbPool>,
        auth_mgr: web::Data<AuthManager>,
        bearer_auth: BearerAuth,
    ) -> Result<Self, MyError> {
        let mut conn = db.get().map_err(|_| MyError::InternalServerError)?;
        let authed_user = AuthedUser::from_bearer_token(conn, auth_mgr, bearer_auth).await?;
        conn = db.get().map_err(|_| MyError::InternalServerError)?;
        Self::from_user(conn, authed_user.user).await
    }

    pub async fn from_user(conn: DbPoolConnection, user: User) -> Result<Self, MyError> {
        let user_data = web::block(move || selectors::admins::UserData::from_user(&conn, user))
            .await
            .map_err(|_| MyError::InternalServerError)?
            .map_err(|_| MyError::UserDoesNotExists)?;
        Ok(Self::from_user_data(&user_data))
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
    web::block(move || services::users::add_user(&conn, &input_user.username, &input_user.password))
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
    let mut conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user = add_user(conn, input_user.into_inner()).await?;
    conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user_data = UserData::from_user(conn, user).await?;
    Ok(web::Json(user_data))
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
    web::block(move || selectors::users::get_user_by_username(&conn, &user_name))
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
    let password = input_user.password.as_bytes();
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user = get_user_by_username(conn, input_user.username.clone()).await?;
    if utils::validate_password(&password, &user.password_hash) {
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
pub struct AuthedUser {
    pub user_id: i32,
    pub user: User,
}

impl AuthedUser {
    async fn from_token(
        conn: DbPoolConnection,
        auth_mgr: web::Data<AuthManager>,
        token: String,
    ) -> Result<Self, MyError> {
        let user_id = web::block(move || auth_mgr.extract_claim::<i32>(&token))
            .await
            .map_err(|_| MyError::InternalServerError)?
            .map_err(|_| MyError::TokenValidationError)?;

        let user = web::block(move || selectors::users::get_user_by_user_id(&conn, user_id))
            .await
            .map_err(|_| MyError::InternalServerError)?
            .map_err(|_| MyError::UserDoesNotExists)?;

        Ok(AuthedUser {
            user_id: user_id,
            user: user,
        })
    }

    pub async fn from_bearer_token(
        conn: DbPoolConnection,
        auth_mgr: web::Data<AuthManager>,
        bearer_auth: BearerAuth,
    ) -> Result<Self, MyError> {
        Self::from_token(conn, auth_mgr, bearer_auth.token().to_string()).await
    }
}

#[get("users/validate_token")]
async fn validate_token(
    bearer_auth: BearerAuth,
    auth_mgr: web::Data<AuthManager>,
    db: web::Data<DbPool>,
) -> actix_web::Result<String, MyError> {
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;

    let _: AuthedUser =
        AuthedUser::from_bearer_token(conn, auth_mgr, bearer_auth).await?;

    Ok("true".to_string())
}

#[get("users/get_user")]
async fn get_user(
    bearer_auth: BearerAuth,
    auth_mgr: web::Data<AuthManager>,
    db: web::Data<DbPool>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    let mut conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let authed_user: AuthedUser =
        AuthedUser::from_bearer_token(conn, auth_mgr, bearer_auth).await?;
    conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user_data = UserData::from_user(conn, authed_user.user).await?;
    Ok(web::Json(user_data))
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
    web::block(move || services::users::update_user_password(&conn, user_id, &new_password))
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
        AuthedUser::from_bearer_token(conn, auth_mgr, bearer_auth).await?;

    conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user = update_user_password(conn, authed_user.user_id, new_password).await?;

    conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let user_data = UserData::from_user(conn, user).await?;

    Ok(web::Json(user_data))
}
