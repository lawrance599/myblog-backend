-- Add migration script here
CREATE TABLE post  (
    id SERIAL PRIMARY KEY,
    tags JSONB NOT NULL,
    first_publish TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_modify TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    query_count INTEGER NOT NULL DEFAULT 0
);