use crate::{
    config::DbPoolConnection,
    models::{admins::Admin, users::User},
    schema::admins::dsl::*,
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
        let user = get_user_by_user_id(conn, userid)?;
        Self::from_user(conn, user)
    }

    pub fn from_user(conn: &DbPoolConnection, user: User) -> Result<Self, diesel::result::Error> {
        use UserData::*;
        match get_admin_by_user_id(conn, user.id) {
            Err(_) => Ok(SimpleUser(user)),
            Ok(admin) => Ok(AdminUser(user, admin.is_super_user)),
        }
    }
}
