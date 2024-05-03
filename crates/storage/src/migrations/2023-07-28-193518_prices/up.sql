CREATE TABLE prices (
    id VARCHAR(256) PRIMARY KEY NOT NULL,
    price float NOT NULL DEFAULT 0,
    price_change_percentage_24h float NOT NULL DEFAULT 0,
    market_cap float NOT NULL DEFAULT 0,
    market_cap_rank INTEGER NOT NULL DEFAULT 0,
    total_volume float NOT NULL DEFAULT 0,
    circulating_supply float NOT NULL DEFAULT 0,
    total_supply float NOT NULL DEFAULT 0,
    max_supply float NOT NULL DEFAULT 0,
    last_updated_at timestamp NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('prices');