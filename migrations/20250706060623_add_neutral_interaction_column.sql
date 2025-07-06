-- Add migration script here

-- -- migrate:up
ALTER TABLE user_interaction
ADD COLUMN num_neutral INTEGER DEFAULT 0;
