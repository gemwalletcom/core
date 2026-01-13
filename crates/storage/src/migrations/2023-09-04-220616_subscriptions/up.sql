CREATE TABLE subscriptions (
    id SERIAL PRIMARY KEY,
    device_id INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    chain VARCHAR NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    address VARCHAR(256) NOT NULL,
    created_at timestamp NOT NULL DEFAULT current_timestamp,
    wallet_index INTEGER NOT NULL,
    UNIQUE(device_id, wallet_index, chain, address)
);

CREATE INDEX subscriptions_address_idx ON subscriptions (address DESC);
CREATE INDEX subscriptions_chain_idx ON subscriptions (chain DESC);

CREATE TABLE subscriptions_addresses_exclude (
    address VARCHAR(128) PRIMARY KEY NOT NULL,
    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    name VARCHAR(64),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('subscriptions_addresses_exclude');

CREATE INDEX subscriptions_addresses_exclude_address_idx ON subscriptions_addresses_exclude (address);
