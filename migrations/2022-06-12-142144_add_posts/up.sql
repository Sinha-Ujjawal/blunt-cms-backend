-- Your SQL goes here
CREATE TABLE posts (
    id SERIAL NOT NULL PRIMARY KEY,
    post_subject TEXT NOT NULL,
    post_body TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
)
