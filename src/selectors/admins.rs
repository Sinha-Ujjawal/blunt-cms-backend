use crate::{
    config::DbPoolConnection, models::{users::User, admins::Admin}, schema::admins::dsl::*,
    selectors::users::get_user_by_user_id,
};

use diesel::prelude::*;

pub fn get_admin_by_user_id(
    conn: &DbPoolConnection,
    userid: i32,
) -> Result<Admin, diesel::result::Error> {
    admins.filter(user_id.eq(&userid)).first::<Admin>(conn)
}

pub enum UserData {
    SimpleUser(User),
    AdminUser(User, bool),
}

impl UserData {
    pub fn from_user_id(
        conn: &DbPoolConnection,
        userid: i32,
    ) -> Result<Self, diesel::result::Error> {
        use UserData::*;
        let user = get_user_by_user_id(conn, userid)?;
        match get_admin_by_user_id(conn, userid) {
            Err(_) => Ok(SimpleUser(user)),
            Ok(admin) => Ok(AdminUser(user, admin.is_super_user))
        }
    }
}
