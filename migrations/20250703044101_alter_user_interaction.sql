-- Add migration script here

-- migrate:up
ALTER TABLE user_interaction
ADD CONSTRAINT user_interaction_user_id_unique UNIQUE (user_id);