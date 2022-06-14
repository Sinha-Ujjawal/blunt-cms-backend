use crate::{
    config::DbPoolConnection,
    models::drafts::{Draft, NewDraft},
};

use diesel::prelude::*;

pub fn add_draft<'a>(
    conn: &DbPoolConnection,
    subject: &'a str,
    body: &'a str,
    postid: Option<i32>,
) -> Result<Draft, diesel::result::Error> {
    use crate::schema::drafts::dsl::*;

    let new_draft = NewDraft {
        draft_subject: subject,
        draft_body: body,
        post_id: postid,
    };

    let res = diesel::insert_into(drafts)
        .values(&new_draft)
        .get_result(conn)?;
    Ok(res)
}
