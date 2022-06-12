-- Your SQL goes here
CREATE TABLE admins (
    id SERIAL NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    is_super_user BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT user_id_fk FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
)
