CREATE TABLE assets (
    id VARCHAR(128) PRIMARY KEY,
    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    token_id VARCHAR(128),
    asset_type VARCHAR(16) NOT NULL REFERENCES assets_types (id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    symbol VARCHAR(16) NOT NULL,
    decimals INTEGER NOT NULL,

    rank INTEGER NOT NULL DEFAULT 0,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(id)
);

SELECT diesel_manage_updated_at('assets');

INSERT INTO "assets" ("id", "chain", "asset_type", "name", "symbol", "decimals") VALUES 
    ('bitcoin', 'bitcoin', 'NATIVE', 'Bitcoin', 'BTC', 8), 
    ('ethereum', 'ethereum', 'NATIVE', 'Ethereum', 'ETH', 18),
    ('binance', 'binance', 'NATIVE', 'BNB Chain', 'BNB', 8),
    ('polygon', 'polygon', 'NATIVE', 'Matic Token', 'MATIC', 18),
    ('ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', 'ethereum', 'ERC20', 'USDC', 'USDC', 6);