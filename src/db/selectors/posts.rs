use crate::{db::actor::DbActor, db::models::posts::Post};
use actix::{Handler, Message};
use diesel::prelude::*;

#[derive(Message)]
#[rtype(result = "Result<Vec<Post>, diesel::result::Error>")]
pub enum GetPosts {
    GetPosts,
}

impl Handler<GetPosts> for DbActor {
    type Result = Result<Vec<Post>, diesel::result::Error>;

    fn handle(&mut self, _: GetPosts, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;
        posts.get_results::<Post>(&conn)
    }
}
