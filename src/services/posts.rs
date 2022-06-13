use crate::{
    config::DbPoolConnection,
    models::posts::{NewPost, Post},
    schema::posts::dsl::*,
};

use diesel::prelude::*;

pub fn add_post<'a>(
    conn: &DbPoolConnection,
    subject: &'a str,
    body: &'a str,
) -> Result<Post, diesel::result::Error> {
    let new_post = NewPost {
        post_subject: subject,
        post_body: body,
    };

    let res = diesel::insert_into(posts)
        .values(&new_post)
        .get_result(conn)?;
    Ok(res)
}
