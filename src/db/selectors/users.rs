use crate::{db::actor::DbActor, db::models::users::User};
use actix::{Handler, Message};
use diesel::prelude::*;

#[derive(Message)]
#[rtype(result = "Result<User, diesel::result::Error>")]
pub struct GetUserByUsername {
    pub username: String,
}

impl Handler<GetUserByUsername> for DbActor {
    type Result = Result<User, diesel::result::Error>;

    fn handle(&mut self, msg: GetUserByUsername, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::users::dsl::*;
        users.filter(username.eq(msg.username)).first::<User>(&conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, diesel::result::Error>")]
pub struct GetUserByUserId {
    pub user_id: i32,
}

impl Handler<GetUserByUserId> for DbActor {
    type Result = Result<User, diesel::result::Error>;

    fn handle(&mut self, msg: GetUserByUserId, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::users::dsl::*;
        users.filter(id.eq(msg.user_id)).first::<User>(&conn)
    }
}
