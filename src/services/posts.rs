use crate::{
    config::DbPoolConnection,
    models::posts::{NewPost, Post},
};

use diesel::prelude::*;

pub fn add_post<'a>(
    conn: &DbPoolConnection,
    subject: &'a str,
    body: &'a str,
) -> Result<Post, diesel::result::Error> {
    use crate::schema::posts::dsl::*;

    let new_post = NewPost {
        post_subject: subject,
        post_body: body,
    };

    let res = diesel::insert_into(posts)
        .values(&new_post)
        .get_result(conn)?;
    Ok(res)
}

pub fn update_post_subject<'a>(
    conn: &DbPoolConnection,
    post_id: i32,
    new_subject: &'a str,
) -> Result<Post, diesel::result::Error> {
    use crate::schema::posts::dsl::*;

    let res = diesel::update(posts)
        .filter(id.eq(post_id))
        .set(post_subject.eq(new_subject))
        .get_result(conn)?;
    Ok(res)
}

pub fn update_post_body<'a>(
    conn: &DbPoolConnection,
    post_id: i32,
    new_body: &'a str,
) -> Result<Post, diesel::result::Error> {
    use crate::schema::posts::dsl::*;

    let res = diesel::update(posts)
        .filter(id.eq(post_id))
        .set(post_body.eq(new_body))
        .get_result(conn)?;
    Ok(res)
}

pub fn delete_post<'a>(
    conn: &DbPoolConnection,
    post_id: i32,
) -> Result<Post, diesel::result::Error> {
    use crate::schema::posts::dsl::*;

    let res = diesel::delete(posts.filter(id.eq(post_id))).get_result(conn)?;
    Ok(res)
}
