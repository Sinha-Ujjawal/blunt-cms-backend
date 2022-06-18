-- Your SQL goes here
CREATE TABLE posts (
    id SERIAL NOT NULL PRIMARY KEY,
    post_subject TEXT NOT NULL,
    post_body TEXT NOT NULL,
    published_status VARCHAR(255) NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT owner_id_fkey FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
)
