use crate::{config::DbPoolConnection, models::posts::Post};

use diesel::prelude::*;

pub fn get_posts<'a>(conn: &DbPoolConnection) -> Result<Vec<Post>, diesel::result::Error> {
    use crate::schema::posts::dsl::*;
    posts.get_results::<Post>(conn)
}
