use crate::{config::DbPoolConnection, models::posts::Post, schema::posts::dsl::*};

use diesel::prelude::*;

pub fn get_posts<'a>(conn: &DbPoolConnection) -> Result<Vec<Post>, diesel::result::Error> {
    posts.get_results::<Post>(conn)
}
