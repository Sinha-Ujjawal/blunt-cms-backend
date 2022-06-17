use crate::db::schema::posts;

use serde::{Deserialize, Serialize};

#[derive(Debug, Identifiable, Serialize, Deserialize, Queryable)]
#[table_name = "posts"]
pub struct Post {
    pub id: i32,
    pub post_subject: String,
    pub post_body: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


#[derive(Insertable, Debug)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub post_subject: &'a str,
    pub post_body: &'a str,
}
