table! {
    admins (id) {
        id -> Int4,
        user_id -> Int4,
        is_super_user -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(admins -> users (user_id));

allow_tables_to_appear_in_same_query!(
    admins,
    users,
);
