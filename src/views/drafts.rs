use crate::{
    db::{actor::DbActor, models::drafts::Draft, services},
    errors::MyError,
    views, AppState,
};
use actix::Addr;
use actix_web::{post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};
use utoipa::Component;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_draft);
}

#[derive(Serialize, Deserialize, Component)]
pub struct DraftData {
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

#[derive(Serialize, Deserialize, Component)]
pub struct CreateDraftData {
    subject: String,
    body: String,
    post_id: Option<i32>,
}

async fn add_draft(
    db_actor_addr: Addr<DbActor>,
    subject: String,
    body: String,
    post_id: Option<i32>,
) -> actix_web::Result<Draft, MyError> {
    db_actor_addr
        .send(services::drafts::AddDraft {
            subject: subject,
            body: body,
            post_id: post_id,
        })
        .await
        .map_err(|_| MyError::InternalServerError)?
        .map_err(|err| MyError::DieselError(err))
}

#[utoipa::path(
    request_body=CreateDraftData,
    responses(
        (status = 200, description = "Create a draft", body = DraftData)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/drafts/create")]
async fn create_draft(
    bearer_auth: BearerAuth,
    draft_data: web::Json<CreateDraftData>,
    app_state: web::Data<AppState>,
) -> actix_web::Result<web::Json<DraftData>, MyError> {
    let db_actor_addr = app_state.get_ref().db_actor_addr.clone();
    let draft_data = draft_data.into_inner();
    let _: views::users::UserData = views::admins::ensure_admin(bearer_auth, app_state).await?;
    let draft = add_draft(
        db_actor_addr,
        draft_data.subject,
        draft_data.body,
        draft_data.post_id,
    )
    .await?;
    Ok(web::Json(DraftData::from_draft(&draft)))
}
