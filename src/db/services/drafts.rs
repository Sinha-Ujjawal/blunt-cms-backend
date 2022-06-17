use crate::{
    db::actor::DbActor,
    db::models::drafts::{Draft, NewDraft},
};
use actix::{Handler, Message};
use diesel::prelude::*;

#[derive(Message)]
#[rtype(result = "Result<Draft, diesel::result::Error>")]
pub struct AddDraft {
    pub subject: String,
    pub body: String,
    pub post_id: Option<i32>,
}

impl Handler<AddDraft> for DbActor {
    type Result = Result<Draft, diesel::result::Error>;

    fn handle(&mut self, msg: AddDraft, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn();
        use crate::db::schema::drafts::dsl::*;

        let new_draft = NewDraft {
            draft_subject: &msg.subject,
            draft_body: &msg.body,
            post_id: msg.post_id,
        };

        let res = diesel::insert_into(drafts)
            .values(&new_draft)
            .get_result(&conn)?;
        Ok(res)
    }
}
