use crate::{
    config::{auth::AuthManager, DbPool, DbPoolConnection},
    errors::MyError,
    models::posts::Post,
    selectors, services, views,
};

use actix_web::{get, post, web, Either};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_post)
        .service(get_posts)
        .service(update_post)
        .service(delete_post);
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
    let _ = views::admins::ensure_admin(bearer_auth, db, auth_mgr).await?;
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

#[derive(Serialize, Deserialize)]
struct UpdatePostSubject {
    new_subject: String,
}

#[derive(Serialize, Deserialize)]
struct UpdatePostBody {
    new_body: String,
}

type UpdatePostData = Either<web::Json<UpdatePostSubject>, web::Json<UpdatePostBody>>;

async fn update_post_subject(
    conn: DbPoolConnection,
    post_id: i32,
    new_subject: String,
) -> actix_web::Result<Post, MyError> {
    web::block(move || services::posts::update_post_subject(&conn, post_id, &new_subject))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

async fn update_post_body(
    conn: DbPoolConnection,
    post_id: i32,
    new_body: String,
) -> actix_web::Result<Post, MyError> {
    web::block(move || services::posts::update_post_body(&conn, post_id, &new_body))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[post("/posts/update/{post_id}")]
async fn update_post(
    path: web::Path<i32>,
    bearer_auth: BearerAuth,
    db: web::Data<DbPool>,
    auth_mgr: web::Data<AuthManager>,
    new_post_data: UpdatePostData,
) -> actix_web::Result<web::Json<PostData>, MyError> {
    let post_id = path.into_inner();
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let _ = views::admins::ensure_admin(bearer_auth, db, auth_mgr).await?;
    let post = match new_post_data {
        Either::Left(web::Json(UpdatePostSubject { new_subject })) => {
            update_post_subject(conn, post_id, new_subject).await?
        }
        Either::Right(web::Json(UpdatePostBody { new_body })) => {
            update_post_body(conn, post_id, new_body).await?
        }
    };

    Ok(web::Json(PostData::from_post(&post)))
}

#[post("/posts/delete/{post_id}")]
async fn delete_post(
    path: web::Path<i32>,
    bearer_auth: BearerAuth,
    db: web::Data<DbPool>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<String, MyError> {
    let post_id = path.into_inner();
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let _ = views::admins::ensure_admin(bearer_auth, db, auth_mgr).await?;
    web::block(move || {
        let _ = services::posts::delete_post(&conn, post_id)?;
        Ok("Success!".to_string())
    })
    .await
    .map_err(|_| MyError::InternalServerError)?
    .map_err(|err| MyError::DieselError(err))
}
