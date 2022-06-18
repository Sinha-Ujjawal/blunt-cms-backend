use crate::{db::schema::posts, db::models::users::User};
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Display)]
pub enum PublishStatus {
    #[display(fmt = "Published")]
    Published,

    #[display(fmt = "Unpublished")]
    Unpublished,

    #[display(fmt = "Request to admin for publish")]
    RequestToAdminForPublish,
}

#[derive(Debug, Identifiable, Serialize, Deserialize, Queryable, Associations)]
#[table_name = "posts"]
#[belongs_to(parent = User)]
pub struct Post {
    pub id: i32,
    pub post_subject: String,
    pub post_body: String,
    pub published_status: String,
    pub user_id: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


#[derive(Insertable, Debug)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub post_subject: &'a str,
    pub post_body: &'a str,
    pub user_id: i32,
    pub published_status: &'a str,
}
