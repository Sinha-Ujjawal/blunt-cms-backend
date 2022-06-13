use crate::{
    config::{auth::AuthManager, DbPool, DbPoolConnection},
    errors::MyError,
    models::posts::Post,
    selectors, services, views,
};

use actix_web::{get, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_post).service(get_posts);
}

#[derive(Serialize, Deserialize)]
struct PostData {
    id: i32,
    subject: String,
    body: String,
}

impl PostData {
    pub fn from_post(post: &Post) -> Self {
        PostData {
            id: post.id,
            subject: post.post_subject.clone(),
            body: post.post_body.clone(),
        }
    }
}

async fn ensure_super_admin(
    bearer_auth: BearerAuth,
    db: web::Data<DbPool>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<views::users::UserData, MyError> {
    let authed_user: views::users::UserData =
        views::users::UserData::from_bearer_token(db, auth_mgr, bearer_auth).await?;
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
) -> actix_web::Result<web::Json<PostData>, MyError> {
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let _ = ensure_super_admin(bearer_auth, db, auth_mgr).await?;
    let post = add_post(conn, post_data).await?;
    Ok(web::Json(PostData::from_post(&post)))
}

#[get("/posts/get_posts")]
async fn get_posts(
    bearer_auth: BearerAuth,
    db: web::Data<DbPool>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<web::Json<Vec<PostData>>, MyError> {
    let mut conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let _: views::users::AuthedUser =
        views::users::AuthedUser::from_bearer_token(conn, auth_mgr, bearer_auth).await?;
    conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let posts = web::block(move || selectors::posts::get_posts(&conn))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))?;
    Ok(web::Json(
        posts
            .iter()
            .map(|post| PostData::from_post(post))
            .collect::<Vec<PostData>>(),
    ))
}
