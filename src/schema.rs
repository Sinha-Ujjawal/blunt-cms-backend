table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}