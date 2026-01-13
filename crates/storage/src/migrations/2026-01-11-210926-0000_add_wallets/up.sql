CREATE TYPE wallet_type AS ENUM ('phrase', 'single', 'view');

CREATE TABLE wallets (
    id VARCHAR(128) PRIMARY KEY,
    wallet_type wallet_type NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE TABLE wallets_subscriptions (
    id SERIAL PRIMARY KEY,
    wallet_id VARCHAR(128) NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    wallet_index INTEGER NOT NULL,
    chain VARCHAR(32) NOT NULL REFERENCES chains(id) ON DELETE CASCADE,
    address VARCHAR(256) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    UNIQUE(wallet_id, device_id, wallet_index, chain, address)
);

CREATE INDEX wallets_subscriptions_wallet_id_idx ON wallets_subscriptions (wallet_id);
CREATE INDEX wallets_subscriptions_device_id_idx ON wallets_subscriptions (device_id);
CREATE INDEX wallets_subscriptions_address_idx ON wallets_subscriptions (address);
CREATE INDEX wallets_subscriptions_chain_address_idx ON wallets_subscriptions (chain, address);
