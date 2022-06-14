use crate::{
    config::{auth::AuthManager, DbPool, DbPoolConnection},
    errors::MyError,
    models::drafts::Draft,
    selectors, services, views,
};

use actix_web::{post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_draft);
}

#[derive(Serialize, Deserialize)]
struct DraftData {
    id: i32,
    subject: String,
    body: String,
    post_id: Option<i32>,
}

impl DraftData {
    pub fn from_draft(draft: &Draft) -> Self {
        DraftData {
            id: draft.id,
            subject: draft.draft_subject.clone(),
            body: draft.draft_body.clone(),
            post_id: draft.post_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CreateDraftData {
    subject: String,
    body: String,
    post_id: Option<i32>,
}

async fn add_draft(
    conn: DbPoolConnection,
    draft_data: web::Json<CreateDraftData>,
) -> actix_web::Result<Draft, MyError> {
    web::block(move || {
        services::drafts::add_draft(
            &conn,
            &draft_data.subject,
            &draft_data.body,
            draft_data.post_id,
        )
    })
    .await
    .map_err(|_| MyError::InternalServerError)?
    .map_err(|err| MyError::DieselError(err))
}

#[post("/drafts/create")]
async fn create_draft(
    bearer_auth: BearerAuth,
    db: web::Data<DbPool>,
    auth_mgr: web::Data<AuthManager>,
    draft_data: web::Json<CreateDraftData>,
) -> actix_web::Result<web::Json<DraftData>, MyError> {
    let _: views::users::UserData =
        views::admins::ensure_admin(bearer_auth, db.clone(), auth_mgr).await?;
    let conn = db.get().map_err(|_| MyError::InternalServerError)?;
    let draft = add_draft(conn, draft_data).await?;
    Ok(web::Json(DraftData::from_draft(&draft)))
}
