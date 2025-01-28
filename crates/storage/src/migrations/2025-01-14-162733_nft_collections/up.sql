CREATE TABLE nft_collections (
    id VARCHAR(512) PRIMARY KEY NOT NULL,

    chain VARCHAR(64) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    
    name VARCHAR(256) NOT NULL,
    description VARCHAR(1024) NOT NULL,
    symbol VARCHAR(128),
    url VARCHAR(256),

    owner VARCHAR(128),
    contract_address VARCHAR(128) NOT NULL,

    image_url VARCHAR(512),

    is_verified BOOLEAN NOT NULL default false,
    is_enabled BOOLEAN NOT NULL default true,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_collections');

