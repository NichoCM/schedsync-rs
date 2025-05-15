-- Your SQL goes here
CREATE TABLE oauth_integrations (
    id SERIAL PRIMARY KEY,
    service SMALLINT NOT NULL,
    integration_id SERIAL NOT NULL UNIQUE REFERENCES apps(id) ON DELETE CASCADE ON UPDATE CASCADE,
    expires_at TIMESTAMP NOT NULL,
    access_token VARCHAR(255) NOT NULL,
    refresh_token VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)