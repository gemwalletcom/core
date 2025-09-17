CREATE TABLE support (
    id SERIAL PRIMARY KEY,
    support_id VARCHAR(32) NOT NULL UNIQUE,
    device_id INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    unread INTEGER NOT NULL DEFAULT 0,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE (device_id, support_id)
);

SELECT diesel_manage_updated_at('support');
