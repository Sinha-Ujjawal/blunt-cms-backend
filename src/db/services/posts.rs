use crate::{
    db::actor::DbActor,
    db::models::posts::{NewPost, Post},
};
use actix::{Handler, Message};
use diesel::prelude::*;

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct AddPost {
    pub subject: String,
    pub body: String,
}

impl Handler<AddPost> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: AddPost, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;

        let new_post = NewPost {
            post_subject: &msg.subject,
            post_body: &msg.body,
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
}

impl Handler<UpdatePostSubject> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: UpdatePostSubject, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;

        let res = diesel::update(posts)
            .filter(id.eq(msg.post_id))
            .set(post_subject.eq(msg.new_subject))
            .get_result(&conn)?;
        Ok(res)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct UpdatePostBody {
    pub post_id: i32,
    pub new_body: String,
}

impl Handler<UpdatePostBody> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: UpdatePostBody, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;

        let res = diesel::update(posts)
            .filter(id.eq(msg.post_id))
            .set(post_body.eq(msg.new_body))
            .get_result(&conn)?;
        Ok(res)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Post, diesel::result::Error>")]
pub struct DeletePost {
    pub post_id: i32,
}

impl Handler<DeletePost> for DbActor {
    type Result = Result<Post, diesel::result::Error>;

    fn handle(&mut self, msg: DeletePost, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::posts::dsl::*;

        let res = diesel::delete(posts.filter(id.eq(msg.post_id))).get_result(&conn)?;
        Ok(res)
    }
}
