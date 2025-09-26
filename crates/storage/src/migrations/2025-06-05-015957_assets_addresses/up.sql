CREATE TABLE assets_addresses
(
    id             SERIAL PRIMARY KEY,
    chain          VARCHAR(32)  NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    asset_id       VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    address        VARCHAR(256) NOT NULL,
    value          VARCHAR(256),
    updated_at     timestamp    NOT NULL default current_timestamp,
    created_at     timestamp    NOT NULL default current_timestamp,
    UNIQUE (asset_id, address)
);

CREATE INDEX assets_addresses_chain_idx ON assets_addresses (chain);
CREATE INDEX assets_addresses_asset_id_idx ON assets_addresses (asset_id);
CREATE INDEX assets_addresses_address_idx ON assets_addresses (address);

SELECT diesel_manage_updated_at('assets_addresses');