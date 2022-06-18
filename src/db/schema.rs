table! {
    drafts (id) {
        id -> Int4,
        draft_subject -> Text,
        draft_body -> Text,
        post_id -> Nullable<Int4>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    posts (id) {
        id -> Int4,
        post_subject -> Text,
        post_body -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_hash -> Varchar,
        is_admin -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(drafts -> posts (post_id));

allow_tables_to_appear_in_same_query!(
    drafts,
    posts,
    users,
);