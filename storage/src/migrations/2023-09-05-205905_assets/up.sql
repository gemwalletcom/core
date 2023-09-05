CREATE TABLE assets (
    id VARCHAR(128) PRIMARY KEY,
    chain VARCHAR(16) NOT NULL,
    name VARCHAR(64) NOT NULL,
    symbol VARCHAR(16) NOT NULL,
    decimals INTEGER NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(id)
);

SELECT diesel_manage_updated_at('assets');

INSERT INTO "assets" ("id", "chain", "name", "symbol", "decimals") VALUES 
    ('bitcoin', 'bitcoin', 'Bitcoin', 'BTC', 8), 
    ('ethereum', 'ethereum', 'Ethereum', 'ETH', 18),
    ('binance', 'binance', 'BNB Chain', 'BNB', 8);