use crate::{
    config::{auth::AuthManager, DbPool},
    errors::MyError,
    views,
};

use actix_web::web;
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn ensure_admin(
    bearer_auth: BearerAuth,
    db: web::Data<DbPool>,
    auth_mgr: web::Data<AuthManager>,
) -> actix_web::Result<views::users::UserData, MyError> {
    let authed_user: views::users::UserData =
        views::users::UserData::from_bearer_token(db, auth_mgr, bearer_auth).await?;
    if authed_user.is_admin {
        Ok(authed_user)
    } else {
        Err(MyError::NotAdmin)
    }
}
