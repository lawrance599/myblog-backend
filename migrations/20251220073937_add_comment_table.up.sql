-- Add up migration script here
CREATE TABLE comment (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL,
    author VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    parent_id INTEGER
);
CREATE INDEX comment_post_id_idx ON comment (post_id);
CREATE INDEX comment_parent_id_idx ON comment (parent_id);