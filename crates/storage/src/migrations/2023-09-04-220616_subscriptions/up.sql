CREATE TABLE subscriptions (
    id SERIAL PRIMARY KEY,
    device_id INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    wallet_index INTEGER NOT NULL,
    chain VARCHAR NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    address VARCHAR(256) NOT NULL,
    updated_at timestamp NOT NULL DEFAULT current_timestamp,
    created_at timestamp NOT NULL DEFAULT current_timestamp,
    UNIQUE(device_id, wallet_index, chain, address)
);

SELECT diesel_manage_updated_at('subscriptions');

CREATE INDEX subscriptions_address_idx ON subscriptions (address DESC);
CREATE INDEX subscriptions_chain_idx ON subscriptions (chain DESC);
