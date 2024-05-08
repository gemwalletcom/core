CREATE TABLE subscriptions_addresses_exclude (
    address VARCHAR(128) PRIMARY KEY NOT NULL,
    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    name VARCHAR(64),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('subscriptions_addresses_exclude');

CREATE INDEX subscriptions_addresses_exclude_address_idx ON subscriptions_addresses_exclude (address);