use crate::{
    argon2_password_hasher::hash_password,
    db::actor::DbActor,
    db::models::users::{NewUser, User},
};
use actix::{Handler, Message};
use diesel::expression::dsl::now;
use diesel::prelude::*;

#[derive(Message)]
#[rtype(result = "Result<User, diesel::result::Error>")]
pub struct AddUser {
    pub username: String,
    pub password: String,
}

impl Handler<AddUser> for DbActor {
    type Result = Result<User, diesel::result::Error>;

    fn handle(&mut self, msg: AddUser, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::users::dsl::*;
        let new_user = NewUser {
            username: &msg.username,
            password_hash: &hash_password(msg.password.as_bytes()),
            is_admin: false, // all new users are not admin by default
        };
        let res = diesel::insert_into(users)
            .values(&new_user)
            .get_result(&conn)?;
        Ok(res)
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, diesel::result::Error>")]
pub struct UpdateUserPassword {
    pub user_id: i32,
    pub new_password: String,
}

impl Handler<UpdateUserPassword> for DbActor {
    type Result = Result<User, diesel::result::Error>;

    fn handle(&mut self, msg: UpdateUserPassword, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(msg.user_id)))
            .set((
                password_hash.eq(&hash_password(msg.new_password.as_bytes())),
                updated_at.eq(now),
            ))
            .get_result(&conn)
    }
}
