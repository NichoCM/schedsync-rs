-- Your SQL goes here
CREATE TABLE oauth2_states (
    id SERIAL PRIMARY KEY,
    state TEXT NOT NULL UNIQUE,
    group_id INT NOT NULL REFERENCES groups(id) ON DELETE CASCADE ON UPDATE CASCADE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)