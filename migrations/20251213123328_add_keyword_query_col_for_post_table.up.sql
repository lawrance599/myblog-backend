-- Add up migration script here
ALTER TABLE post ADD COLUMN kw tsvector NOT NULL;
CREATE INDEX post_kw_idx ON post USING GIN (keyword_query);