CREATE TABLE subscriptions (
    id SERIAL PRIMARY KEY,
    device_id INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    chain VARCHAR NOT NULL,
    address VARCHAR(256) NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(device_id, chain, address)
);

SELECT diesel_manage_updated_at('subscriptions');