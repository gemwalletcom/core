CREATE TABLE fiat_assets (
  id SERIAL PRIMARY KEY,
  asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
  provider VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
  symbol VARCHAR NOT NULL,
  network VARCHAR NULL,
  enabled BOOLEAN NOT NULL DEFAULT TRUE,
  updated_at timestamp NOT NULL default current_timestamp,
  created_at timestamp NOT NULL default current_timestamp,
  UNIQUE NULLS NOT DISTINCT(asset_id, provider, symbol, network)
);

SELECT diesel_manage_updated_at('fiat_assets');