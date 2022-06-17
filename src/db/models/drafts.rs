use crate::{db::models::posts::Post, db::schema::drafts};

use serde::{Deserialize, Serialize};

#[derive(Debug, Identifiable, Associations, Queryable, Serialize, Deserialize)]
#[belongs_to(Post)]
#[table_name = "drafts"]
pub struct Draft {
    pub id: i32,
    pub draft_subject: String,
    pub draft_body: String,
    pub post_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "drafts"]
pub struct NewDraft<'a> {
    pub draft_subject: &'a str,
    pub draft_body: &'a str,
    pub post_id: Option<i32>,
}
