CREATE TABLE scan_addresses_types (
    id VARCHAR(32) PRIMARY KEY NOT NULL
);

CREATE TABLE scan_addresses (
    id SERIAL PRIMARY KEY,
    chain VARCHAR NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    address VARCHAR(128) NOT NULL,
    name VARCHAR(128),
    type VARCHAR(32) REFERENCES scan_addresses_types (id) ON DELETE CASCADE,
    is_verified boolean NOT NULL DEFAULT false,
    is_fraudulent boolean NOT NULL DEFAULT false,
    is_memo_required boolean NOT NULL DEFAULT false,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(chain, address)
);

SELECT diesel_manage_updated_at('scan_addresses');

CREATE INDEX scan_addresses_address_idx ON scan_addresses (address);
CREATE INDEX scan_addresses_chain_idx ON scan_addresses (chain);
CREATE INDEX scan_addresses_type_idx ON scan_addresses (type);
