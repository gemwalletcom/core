CREATE TABLE devices (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(32) NOT NULL,
    is_push_enabled boolean NOT NULL,
    platform VARCHAR(8) NOT NULL,
    token VARCHAR(256) NOT NULL,
    locale VARCHAR(8) NOT NULL,
    version VARCHAR(8) NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(device_id)
);

CREATE INDEX devices_token_idx ON devices (token);

SELECT diesel_manage_updated_at('devices');