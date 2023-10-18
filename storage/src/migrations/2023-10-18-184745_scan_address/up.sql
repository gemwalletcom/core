CREATE TABLE scan_addresses (
    id SERIAL PRIMARY KEY,
    address VARCHAR(128) NOT NULL,
    name VARCHAR(64),
    type VARCHAR(32),
    is_verified boolean NOT NULL DEFAULT false,
    is_fradulent boolean NOT NULL DEFAULT false,
    is_memo_required boolean NOT NULL DEFAULT false,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);