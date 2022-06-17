use crate::{
    db::{actor::DbActor, models::posts::Post, selectors, services},
    errors::MyError,
    views, AppState,
};
use actix::Addr;
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
    db_actor_addr: Addr<DbActor>,
    subject: String,
    body: String,
) -> actix_web::Result<Post, MyError> {
    db_actor_addr
        .send(services::posts::AddPost {
            subject: subject,
            body: body,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[post("/posts/create")]
async fn create_post(
    bearer_auth: BearerAuth,
    post_data: web::Json<CreatePostData>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<PostData>, MyError> {
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let post_data = post_data.into_inner();
    let _ = views::admins::ensure_admin(bearer_auth, app_state).await?;
    let post = add_post(db_actor_addr, post_data.subject, post_data.body).await?;
    Ok(web::Json(PostData::from_post(&post)))
}

#[get("/posts/get_posts")]
async fn get_posts(
    bearer_auth: BearerAuth,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<Vec<PostData>>, MyError> {
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.get_ref().auth_mgr_addr.clone();
    let _: views::users::AuthedUser = views::users::AuthedUser::from_bearer_token(
        db_actor_addr.clone(),
        auth_mgr_addr,
        bearer_auth,
    )
    .await?;
    let posts = db_actor_addr
        .send(selectors::posts::GetPosts::GetPosts)
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
    db_actor_addr: Addr<DbActor>,
    post_id: i32,
    new_subject: String,
) -> actix_web::Result<Post, MyError> {
    db_actor_addr
        .send(services::posts::UpdatePostSubject {
            post_id: post_id,
            new_subject: new_subject,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

async fn update_post_body(
    db_actor_addr: Addr<DbActor>,
    post_id: i32,
    new_body: String,
) -> actix_web::Result<Post, MyError> {
    db_actor_addr
        .send(services::posts::UpdatePostBody {
            post_id: post_id,
            new_body: new_body,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[post("/posts/update/{post_id}")]
async fn update_post(
    path: web::Path<i32>,
    bearer_auth: BearerAuth,
    new_post_data: UpdatePostData,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<PostData>, MyError> {
    let post_id = path.into_inner();
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let _ = views::admins::ensure_admin(bearer_auth, app_state).await?;
    let post = match new_post_data {
        Either::Left(web::Json(UpdatePostSubject { new_subject })) => {
            update_post_subject(db_actor_addr, post_id, new_subject).await?
        }
        Either::Right(web::Json(UpdatePostBody { new_body })) => {
            update_post_body(db_actor_addr, post_id, new_body).await?
        }
    };

    Ok(web::Json(PostData::from_post(&post)))
}

#[post("/posts/delete/{post_id}")]
async fn delete_post(
    path: web::Path<i32>,
    bearer_auth: BearerAuth,
    app_state: web::Data<AppState>,
) -> actix_web::Result<String, MyError> {
    let post_id = path.into_inner();
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let _ = views::admins::ensure_admin(bearer_auth, app_state).await?;
    let _ = db_actor_addr
        .send(services::posts::DeletePost { post_id: post_id })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err));
    Ok("Success!".to_string())
}
