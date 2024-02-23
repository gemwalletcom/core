CREATE TABLE fiat_assets (
  id SERIAL PRIMARY KEY,
  asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
  provider VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
  symbol VARCHAR NOT NULL,
  network VARCHAR NULL,
  updated_at timestamp NOT NULL default current_timestamp,
  created_at timestamp NOT NULL default current_timestamp,
  UNIQUE NULLS NOT DISTINCT(asset_id, provider, symbol, network)
);

SELECT diesel_manage_updated_at('fiat_assets');

INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('ethereum', 'moonpay','eth',null);
INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('ethereum', 'transak','ETH','ethereum');
INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('ethereum', 'mercuryo','ETH',null);

INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', 'moonpay','usdc',null);
INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', 'transak','USDC','ethereum');
INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', 'mercuryo','USDC',null);

INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('polygon', 'moonpay','matic_polygon',null);
INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('polygon', 'transak','MATIC','polygon');
INSERT INTO fiat_assets(asset_id, provider, symbol, network) VALUES ('polygon', 'mercuryo','MATIC',null);
