use crate::{
    db::actor::DbActor,
    db::models::posts::{NewPost, Post, PublishStatus},
};
use actix::{Handler, Message};
use diesel::expression::dsl::now;
use diesel::prelude::*;

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct AddPost {
    pub subject: String,
    pub body: String,
    pub user_id: i32,
}

impl Handler<AddPost> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: AddPost, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;

        let new_post = NewPost {
            post_subject: &msg.subject,
            post_body: &msg.body,
            user_id: msg.user_id,
            published_status: &format!("{}", PublishStatus::Unpublished),
        };

        let res = diesel::insert_into(posts)
            .values(&new_post)
            .get_result(&conn)?;
        Ok(res)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct UpdatePostSubject {
    pub post_id: i32,
    pub new_subject: String,
    pub user_id: i32,
}

impl Handler<UpdatePostSubject> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: UpdatePostSubject, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;

        let res = diesel::update(posts)
            .filter(id.eq(msg.post_id).and(user_id.eq(msg.user_id)))
            .set((post_subject.eq(msg.new_subject), updated_at.eq(now)))
            .get_result(&conn)?;
        Ok(res)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct UpdatePostBody {
    pub post_id: i32,
    pub new_body: String,
    pub user_id: i32,
}

impl Handler<UpdatePostBody> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: UpdatePostBody, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;

        let res = diesel::update(posts)
            .filter(id.eq(msg.post_id).and(user_id.eq(msg.user_id)))
            .set((post_body.eq(msg.new_body), updated_at.eq(now)))
            .get_result(&conn)?;
        Ok(res)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct DeletePost {
    pub post_id: i32,
    pub user_id: i32,
}

impl Handler<DeletePost> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: DeletePost, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;

        let res = diesel::delete(posts.filter(id.eq(msg.post_id).and(user_id.eq(msg.user_id))))
            .get_result(&conn)?;
        Ok(res)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct RequestToPublishPost {
    pub post_id: i32,
    pub user_id: i32,
}

impl Handler<RequestToPublishPost> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: RequestToPublishPost, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;
        let res = diesel::update(posts.filter(id.eq(msg.post_id).and(user_id.eq(msg.user_id))))
            .set((
                published_status.eq(format!("{}", PublishStatus::RequestToAdminForPublish)),
                updated_at.eq(now),
            ))
            .get_result(&conn)?;
        Ok(res)
    }
}
