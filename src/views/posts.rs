use crate::{
    db::{
        actor::DbActor,
        models::{posts::Post, users::User},
        selectors, services,
    },
    errors::MyError,
    views, AppState,
};
use actix::Addr;
use actix_web::{get, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use utoipa::Component;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_post)
        .service(get_posts)
        .service(get_drafts)
        .service(update_post_subject_handler)
        .service(update_post_body_handler)
        .service(delete_post)
        .service(request_admin_to_publish)
        .service(publish_post);
}

#[derive(Serialize, Deserialize, Component)]
pub struct PostData {
    id: i32,
    subject: String,
    body: String,
    owner: bool,
    owner_name: String,
    status: String,
}

impl PostData {
    pub fn from_post_data(post: &selectors::posts::PostData, user_maybe: Option<User>) -> Self {
        let owner = match user_maybe {
            Some(user) => user.id == post.user_id,
            _ => false,
        };
        PostData {
            id: post.id,
            subject: post.subject.clone(),
            body: post.body.clone(),
            owner: owner,
            owner_name: post.owner_name.clone(),
            status: post.status.clone(),
        }
    }

    pub fn from_post(post: &Post, user: User) -> Self {
        PostData {
            id: post.id,
            subject: post.post_subject.clone(),
            body: post.post_body.clone(),
            owner: true,
            owner_name: user.username,
            status: post.published_status.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Component)]
pub struct CreatePostData {
    subject: String,
    body: String,
}

async fn add_post(
    db_actor_addr: Addr<DbActor>,
    subject: String,
    body: String,
    user_id: i32,
) -> actix_web::Result<Post, MyError> {
    db_actor_addr
        .send(services::posts::AddPost {
            subject: subject,
            body: body,
            user_id: user_id,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[utoipa::path(
    request_body=CreatePostData,
    responses(
        (status = 200, description = "Create Post", body = PostData)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/posts/create")]
async fn create_post(
    bearer_auth: BearerAuth,
    post_data: web::Json<CreatePostData>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<PostData>, MyError> {
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.get_ref().auth_mgr_addr.clone();
    let post_data = post_data.into_inner();
    let authed_user = views::users::AuthedUser::from_bearer_token(
        db_actor_addr.clone(),
        auth_mgr_addr,
        bearer_auth,
    )
    .await?;
    let post = add_post(
        db_actor_addr,
        post_data.subject,
        post_data.body,
        authed_user.user.id,
    )
    .await?;
    Ok(web::Json(PostData::from_post(&post, authed_user.user)))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Request Admin to publish this post")
    )
)]
#[get("/posts/get_posts")]
async fn get_posts(
    bearer_auth: Option<BearerAuth>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<Vec<PostData>>, MyError> {
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.get_ref().auth_mgr_addr.clone();
    let user_maybe: Option<User> = match bearer_auth {
        None => None,
        Some(bearer_auth) => views::users::AuthedUser::from_bearer_token(
            db_actor_addr.clone(),
            auth_mgr_addr,
            bearer_auth,
        )
        .await
        .ok()
        .map(|authed_user| authed_user.user),
    };
    let posts = db_actor_addr
        .send(selectors::posts::GetPosts::GetPublishedPosts)
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))?;
    Ok(web::Json(
        posts
            .iter()
            .map(|post| PostData::from_post_data(post, user_maybe.clone()))
            .collect::<Vec<PostData>>(),
    ))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Get Drafts", body = [PostData])
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/posts/get_drafts")]
async fn get_drafts(
    bearer_auth: BearerAuth,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<Vec<PostData>>, MyError> {
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.get_ref().auth_mgr_addr.clone();
    let authed_user = views::users::AuthedUser::from_bearer_token(
        db_actor_addr.clone(),
        auth_mgr_addr,
        bearer_auth,
    )
    .await?;
    let unpublished_posts = db_actor_addr
        .send(selectors::posts::GetPosts::GetUnpublishedPosts(
            authed_user.user.id,
        ))
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))?;
    Ok(web::Json(
        unpublished_posts
            .iter()
            .map(|post| PostData::from_post_data(post, Some(authed_user.user.clone())))
            .collect::<Vec<PostData>>(),
    ))
}

async fn ensure_user_owns_post(
    db_actor_addr: Addr<DbActor>,
    user_id: i32,
    post_id: i32,
) -> Result<Post, MyError> {
    let post = db_actor_addr
        .send(selectors::posts::GetPostById { post_id: post_id })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))?;
    if post.user_id == user_id {
        Ok(post)
    } else {
        Err(MyError::YouDontOwnThisPost)
    }
}

#[derive(Serialize, Deserialize, Component)]
pub struct UpdatePostSubject {
    new_subject: String,
}

async fn update_post_subject(
    db_actor_addr: Addr<DbActor>,
    post_id: i32,
    new_subject: String,
    user_id: i32,
) -> actix_web::Result<Post, MyError> {
    db_actor_addr
        .send(services::posts::UpdatePostSubject {
            post_id: post_id,
            new_subject: new_subject,
            user_id: user_id,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[utoipa::path(
    params(
        ("post_id" = i32, path, description = "Post database id"),
    ),
    request_body=UpdatePostSubject,
    responses(
        (status = 200, description = "Update Post Subject", body = PostData)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/posts/update/subject/{post_id}")]
async fn update_post_subject_handler(
    path: web::Path<i32>,
    bearer_auth: BearerAuth,
    new_post_subject: web::Json<UpdatePostSubject>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<PostData>, MyError> {
    let post_id = path.into_inner();
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.get_ref().auth_mgr_addr.clone();
    let new_post_subject = new_post_subject.into_inner();
    let authed_user = views::users::AuthedUser::from_bearer_token(
        db_actor_addr.clone(),
        auth_mgr_addr,
        bearer_auth,
    )
    .await?;
    let _ = ensure_user_owns_post(db_actor_addr.clone(), authed_user.user.id, post_id).await?;
    let post = update_post_subject(
        db_actor_addr,
        post_id,
        new_post_subject.new_subject,
        authed_user.user.id,
    )
    .await?;
    Ok(web::Json(PostData::from_post(&post, authed_user.user)))
}

#[derive(Serialize, Deserialize, Component)]
pub struct UpdatePostBody {
    new_body: String,
}

async fn update_post_body(
    db_actor_addr: Addr<DbActor>,
    post_id: i32,
    new_body: String,
    user_id: i32,
) -> actix_web::Result<Post, MyError> {
    db_actor_addr
        .send(services::posts::UpdatePostBody {
            post_id: post_id,
            new_body: new_body,
            user_id: user_id,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[utoipa::path(
    params(
        ("post_id" = i32, path, description = "Post database id"),
    ),
    request_body=UpdatePostBody,
    responses(
        (status = 200, description = "Update Post Body", body = PostData)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/posts/update/body/{post_id}")]
async fn update_post_body_handler(
    path: web::Path<i32>,
    bearer_auth: BearerAuth,
    new_post_body: web::Json<UpdatePostBody>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<PostData>, MyError> {
    let post_id = path.into_inner();
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.get_ref().auth_mgr_addr.clone();
    let new_post_body = new_post_body.into_inner();
    let authed_user = views::users::AuthedUser::from_bearer_token(
        db_actor_addr.clone(),
        auth_mgr_addr,
        bearer_auth,
    )
    .await?;
    let _ = ensure_user_owns_post(db_actor_addr.clone(), authed_user.user.id, post_id).await?;
    let post = update_post_body(
        db_actor_addr,
        post_id,
        new_post_body.new_body,
        authed_user.user.id,
    )
    .await?;
    Ok(web::Json(PostData::from_post(&post, authed_user.user)))
}

#[utoipa::path(
    params(
        ("post_id" = i32, path, description = "Post database id"),
    ),
    responses(
        (status = 200, description = "Delete Post", body = String)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/posts/delete/{post_id}")]
async fn delete_post(
    path: web::Path<i32>,
    bearer_auth: BearerAuth,
    app_state: web::Data<AppState>,
) -> actix_web::Result<String, MyError> {
    let post_id = path.into_inner();
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.get_ref().auth_mgr_addr.clone();
    let user = views::users::UserData::from_bearer_token(
        db_actor_addr.clone(),
        auth_mgr_addr,
        bearer_auth,
    )
    .await?;
    let _ = ensure_user_owns_post(db_actor_addr.clone(), user.id, post_id).await?;
    let _ = db_actor_addr
        .send(services::posts::DeletePost {
            post_id: post_id,
            user_id: user.id,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err));
    Ok("Success!".to_string())
}

#[utoipa::path(
    params(
        ("post_id" = i32, path, description = "Post database id"),
    ),
    responses(
        (status = 200, description = "Request To Publish", body = [PostData])
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/posts/request_to_publish/{post_id}")]
async fn request_admin_to_publish(
    bearer_auth: BearerAuth,
    post_id: web::Path<i32>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<String, MyError> {
    let post_id = post_id.into_inner();
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.get_ref().auth_mgr_addr.clone();
    let authed_user: views::users::AuthedUser = views::users::AuthedUser::from_bearer_token(
        db_actor_addr.clone(),
        auth_mgr_addr,
        bearer_auth,
    )
    .await?;
    let _: Post =
        ensure_user_owns_post(db_actor_addr.clone(), authed_user.user.id, post_id).await?;
    let _ = db_actor_addr
        .send(services::posts::RequestToPublishPost {
            post_id: post_id,
            user_id: authed_user.user.id,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|e| MyError::DieselError(e))?;

    Ok("Success".into())
}

#[utoipa::path(
    params(
        ("post_id" = i32, path, description = "Post database id"),
    ),
    responses(
        (status = 200, description = "", body = [PostData])
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/posts/publish/{post_id}")]
async fn publish_post(
    bearer_auth: BearerAuth,
    post_id: web::Path<i32>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<String, MyError> {
    let post_id = post_id.into_inner();
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let _: views::users::AuthedUser =
        views::admins::ensure_admin(bearer_auth, app_state).await?;
    let _ = db_actor_addr
        .send(services::posts::PublishPost { post_id: post_id })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|e| MyError::DieselError(e))?;

    Ok("Success".into())
}
