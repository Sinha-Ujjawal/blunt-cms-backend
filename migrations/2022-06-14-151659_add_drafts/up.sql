-- Your SQL goes here
CREATE TABLE drafts (
    id SERIAL NOT NULL PRIMARY KEY,
    draft_subject TEXT NOT NULL,
    draft_body TEXT NOT NULL,
    post_id INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT post_id_fk FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
)
