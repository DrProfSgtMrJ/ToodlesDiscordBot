-- Add migration script here

-- migrate:up
CREATE TABLE user_interaction (
    id SERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    num_positive INTEGER DEFAULT 0,
    num_negative INTEGER DEFAULT 0,
    last_interaction TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)