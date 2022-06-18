use crate::{
    argon2_password_hasher,
    auth::actor::AuthManager,
    auth::actor::{CreateToken, ExtractClaim},
    db::{actor::DbActor, models::users::User, selectors, services},
    errors::MyError,
    AppState,
};
use actix::Addr;
use actix_web::{get, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use utoipa::Component;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(signup)
        .service(login)
        .service(get_user)
        .service(validate_token)
        .service(change_password);
}

#[derive(Serialize, Deserialize, Component)]
pub struct UserData {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
}

impl UserData {
    pub fn from_user(user: User) -> Self {
        UserData {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
        }
    }

    pub async fn from_bearer_token(
        db_actor_addr: Addr<DbActor>,
        auth_mgr_addr: Addr<AuthManager>,
        bearer_auth: BearerAuth,
    ) -> Result<Self, MyError> {
        let authed_user =
            AuthedUser::from_bearer_token(db_actor_addr, auth_mgr_addr, bearer_auth).await?;
        Ok(Self::from_user(authed_user.user))
    }
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub struct SignUpInput {
    username: String,
    password: String,
}

async fn add_user(
    db_actor_addr: Addr<DbActor>,
    input_user: SignUpInput,
) -> actix_web::Result<User, MyError> {
    db_actor_addr
        .send(services::users::AddUser {
            username: input_user.username,
            password: input_user.password,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| match err {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => MyError::UserAlreadyExists,
            _ => MyError::DieselError(err),
        })
}

#[utoipa::path(
    request_body=SignUpInput,
    responses(
        (status = 200, description = "Sign up", body = UserData)
    )
)]
#[post("/users/signup")]
async fn signup(
    input_user: web::Json<SignUpInput>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    let db_actor_addr = app_state.as_ref().db_actor_addr.clone();
    let user = add_user(db_actor_addr, input_user.into_inner()).await?;
    Ok(web::Json(UserData::from_user(user)))
}

#[derive(Serialize, Deserialize, Component)]
pub struct Token {
    token: String,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub struct LogInInput {
    username: String,
    password: String,
}

async fn get_user_by_username(
    db_actor_addr: Addr<DbActor>,
    username: String,
) -> actix_web::Result<User, MyError> {
    db_actor_addr
        .send(selectors::users::GetUserByUsername { username: username })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|_| MyError::UserDoesNotExists)
}

#[utoipa::path(
    request_body=LogInInput,
    responses(
        (status = 200, description = "Log in", body = Token)
    )
)]
#[post("/users/login")]
async fn login(
    input_user: web::Json<LogInInput>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<Token>, MyError> {
    let db_actor_addr = app_state.as_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.as_ref().auth_mgr_addr.clone();
    let password = input_user.password.as_bytes();
    let user = get_user_by_username(db_actor_addr, input_user.username.clone()).await?;
    if argon2_password_hasher::validate_password(&password, &user.password_hash) {
        let token = auth_mgr_addr
            .send(CreateToken { data: user.id })
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
        db_actor_addr: Addr<DbActor>,
        auth_mgr_addr: Addr<AuthManager>,
        token: String,
    ) -> Result<Self, MyError> {
        let user_id: i32 = auth_mgr_addr
            .send(ExtractClaim {
                token: token,
                phantom: PhantomData::<i32>,
            })
            .await
            .map_err(|_| MyError::InternalServerError)?
            .map_err(|_| MyError::TokenValidationError)?;

        let user = db_actor_addr
            .send(selectors::users::GetUserByUserId { user_id: user_id })
            .await
            .map_err(|_| MyError::InternalServerError)?
            .map_err(|_| MyError::UserDoesNotExists)?;

        Ok(AuthedUser {
            user_id: user_id,
            user: user,
        })
    }

    pub async fn from_bearer_token(
        db_actor_addr: Addr<DbActor>,
        auth_mgr_addr: Addr<AuthManager>,
        bearer_auth: BearerAuth,
    ) -> Result<Self, MyError> {
        Self::from_token(
            db_actor_addr,
            auth_mgr_addr,
            bearer_auth.token().to_string(),
        )
        .await
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Validate token", body = String)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/users/validate_token")]
async fn validate_token(
    bearer_auth: BearerAuth,
    app_state: web::Data<AppState>,
) -> actix_web::Result<String, MyError> {
    let db_actor_addr = app_state.as_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.as_ref().auth_mgr_addr.clone();

    let _: AuthedUser =
        AuthedUser::from_bearer_token(db_actor_addr, auth_mgr_addr, bearer_auth).await?;

    Ok("true".to_string())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Get User data", body = UserData)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/users/get_user")]
async fn get_user(
    bearer_auth: BearerAuth,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    let db_actor_addr = app_state.as_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.as_ref().auth_mgr_addr.clone();
    let authed_user: AuthedUser =
        AuthedUser::from_bearer_token(db_actor_addr, auth_mgr_addr, bearer_auth).await?;
    Ok(web::Json(UserData::from_user(authed_user.user)))
}

#[derive(Serialize, Deserialize, Component)]
pub struct UserChangePasswordInput {
    new_password: String,
}

async fn update_user_password(
    db_actor_addr: Addr<DbActor>,
    user_id: i32,
    new_password: String,
) -> actix_web::Result<User, MyError> {
    db_actor_addr
        .send(services::users::UpdateUserPassword {
            user_id: user_id,
            new_password: new_password,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[utoipa::path(
    request_body=UserChangePasswordInput,
    responses(
        (status = 200, description = "Get User data", body = UserData)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/users/change_password")]
async fn change_password(
    bearer_auth: BearerAuth,
    web::Json(UserChangePasswordInput { new_password }): web::Json<UserChangePasswordInput>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<UserData>, MyError> {
    let db_actor_addr = app_state.as_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.as_ref().auth_mgr_addr.clone();

    let authed_user: AuthedUser =
        AuthedUser::from_bearer_token(db_actor_addr.clone(), auth_mgr_addr, bearer_auth).await?;

    let user = update_user_password(db_actor_addr, authed_user.user_id, new_password).await?;

    Ok(web::Json(UserData::from_user(user)))
}
