-- Add down migration script here
DROP INDEX IF EXISTS comment_post_id_idx;
DROP INDEX IF EXISTS comment_parent_id_idx;
DROP TABLE IF EXISTS comment;