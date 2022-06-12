use crate::{models::users::User, schema::*};

use serde::{Deserialize, Serialize};

#[derive(Debug, Identifiable, Associations, Serialize, Deserialize, Queryable)]
#[belongs_to(User)]
#[table_name = "admins"]
pub struct Admin {
    pub id: i32,
    pub user_id: i32,
    pub is_super_user: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
