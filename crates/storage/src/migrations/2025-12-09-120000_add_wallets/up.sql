CREATE TYPE wallet_type AS ENUM ('multicoin', 'single', 'privateKey', 'view');
CREATE TYPE wallet_source AS ENUM ('create', 'import');

CREATE TABLE wallets (
    id SERIAL PRIMARY KEY,
    identifier VARCHAR(128) UNIQUE NOT NULL,
    wallet_type wallet_type NOT NULL,
    source wallet_source NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE TABLE wallets_addresses (
    id SERIAL PRIMARY KEY,
    address VARCHAR(256) UNIQUE NOT NULL
);

CREATE TABLE wallets_subscriptions (
    id SERIAL PRIMARY KEY,
    wallet_id INTEGER NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    chain VARCHAR(32) NOT NULL REFERENCES chains(id) ON DELETE CASCADE,
    address_id INTEGER NOT NULL REFERENCES wallets_addresses(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    UNIQUE(wallet_id, device_id, chain, address_id)
);

CREATE INDEX wallets_subscriptions_wallet_id_idx ON wallets_subscriptions (wallet_id);
CREATE INDEX wallets_subscriptions_device_id_idx ON wallets_subscriptions (device_id);
CREATE INDEX wallets_subscriptions_address_id_idx ON wallets_subscriptions (address_id);
CREATE INDEX wallets_subscriptions_chain_address_id_idx ON wallets_subscriptions (chain, address_id);
CREATE INDEX wallets_subscriptions_device_chain_idx ON wallets_subscriptions (device_id, chain);
