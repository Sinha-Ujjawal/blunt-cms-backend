use crate::{
    config::{auth::AuthManager, DbPool, DbPoolConnection},
    errors::MyError,
    models::posts::Post,
    selectors, services, views,
};

use actix_web::{post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_post);
}

async fn ensure_super_admin(
    bearer_auth: BearerAuth,
    db: web::Data<DbPool>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<views::users::UserData, MyError> {
    let authed_user: views::users::UserData =
        views::users::UserData::from_bearer_token(
            db,
            auth_mgr,
            bearer_auth.token().to_string(),
        )
        .await?;
    if authed_user.is_super_admin {
        Ok(authed_user)
    } else {
        Err(MyError::NotSuperAdmin)
    }
}

#[derive(Serialize, Deserialize)]
struct CreatePostData {
    subject: String,
    body: String,
}

async fn add_post(
    conn: DbPoolConnection,
    post_data: web::Json<CreatePostData>,
) -> actix_web::Result<Post, MyError> {
    web::block(move || services::posts::add_post(&conn, &post_data.subject, &post_data.body))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[post("/posts/create")]
async fn create_post(
    bearer_auth: BearerAuth,
    db: web::Data<DbPool>,
    auth_mgr: web::Data<AuthManager>,
    post_data: web::Json<CreatePostData>,
) -> actix_web::Result<web::Json<Post>, MyError> {
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let _ = ensure_super_admin(bearer_auth, db, auth_mgr).await?;
    let post = add_post(conn, post_data).await?;
    Ok(web::Json(post))
}
