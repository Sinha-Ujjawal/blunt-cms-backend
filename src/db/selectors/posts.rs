use crate::{
    db::actor::DbActor,
    db::models::posts::{Post, PublishStatus},
};
use actix::{Handler, Message};
use diesel::prelude::*;

#[derive(Queryable)]
pub struct PostData {
    pub id: i32,
    pub subject: String,
    pub body: String,
    pub user_id: i32,
    pub owner_name: String,
    pub status: String,
}

#[derive(Message)]
#[rtype(result = "Result<Vec<PostData>, diesel::result::Error>")]
pub enum GetPosts {
    GetPublishedPosts,
    GetUnpublishedPosts(i32),
}

impl Handler<GetPosts> for DbActor {
    type Result = Result<Vec<PostData>, diesel::result::Error>;

    fn handle(&mut self, msg: GetPosts, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();

        use GetPosts::*;

        use crate::db::schema::{posts, posts::dsl::*, users, users::dsl::*};

        match msg {
            GetPublishedPosts => {
                let status = format!("{}", PublishStatus::Published);
                posts
                    .filter(published_status.eq(&status))
                    .inner_join(users.on(posts::user_id.eq(users::id)))
                    .select((
                        posts::id,
                        posts::post_subject,
                        posts::post_body,
                        users::id,
                        users::username,
                        posts::published_status,
                    ))
                    .get_results::<PostData>(&conn)
            }
            GetUnpublishedPosts(owner_id) => {
                let status = format!("{}", PublishStatus::Unpublished);
                posts
                    .filter(published_status.eq(&status).and(user_id.eq(owner_id)))
                    .inner_join(users.on(posts::user_id.eq(users::id)))
                    .select((
                        posts::id,
                        posts::post_subject,
                        posts::post_body,
                        users::id,
                        users::username,
                        posts::published_status,
                    ))
                    .get_results::<PostData>(&conn)
            }
        }
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
