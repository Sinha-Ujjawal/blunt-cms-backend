use crate::{
    db::actor::DbActor,
    db::models::posts::{Post, PublishStatus},
};
use actix::{Handler, Message};
use diesel::prelude::*;

#[derive(Message)]
#[rtype(result = "Result<Vec<Post>, diesel::result::Error>")]
pub enum GetPosts {
    GetPublishedPosts,
}

impl Handler<GetPosts> for DbActor {
    type Result = Result<Vec<Post>, diesel::result::Error>;

    fn handle(&mut self, msg: GetPosts, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();

        use GetPosts::*;

        let status = match msg {
            GetPublishedPosts => format!("{}", PublishStatus::Published),
        };

        use crate::db::schema::posts::dsl::*;
        posts
            .filter(published_status.eq(&status))
            .get_results::<Post>(&conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct GetPostById {
    pub post_id: i32,
}

impl Handler<GetPostById> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: GetPostById, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;
        posts.filter(id.eq(msg.post_id)).get_result::<Post>(&conn)
    }
}
