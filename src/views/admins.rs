use crate::{errors::MyError, views, AppState};

use actix_web::web;
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn ensure_admin(
    bearer_auth: BearerAuth,
    app_state: web::Data<AppState>,
) -> actix_web::Result<views::users::UserData, MyError> {
    let db_actor_addr = app_state.as_ref().db_actor_addr.clone();
    let auth_mgr_addr = app_state.as_ref().auth_mgr_addr.clone();

    let authed_user: views::users::UserData =
        views::users::UserData::from_bearer_token(db_actor_addr, auth_mgr_addr, bearer_auth)
            .await?;
    if authed_user.is_admin {
        Ok(authed_user)
    } else {
        Err(MyError::NotAdmin)
    }
}
