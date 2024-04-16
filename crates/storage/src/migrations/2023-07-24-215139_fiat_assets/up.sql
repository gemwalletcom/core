CREATE TABLE fiat_assets (
  id VARCHAR(128) PRIMARY KEY,
  asset_id VARCHAR(128) REFERENCES assets (id) ON DELETE CASCADE,
  provider VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
  code VARCHAR(128) NOT NULL,
  symbol VARCHAR(128) NOT NULL,
  network VARCHAR(128) NULL,
  token_id VARCHAR(128) NULL,
  enabled BOOLEAN NOT NULL DEFAULT TRUE,
  hidden BOOLEAN NOT NULL DEFAULT FALSE,
  updated_at timestamp NOT NULL default current_timestamp,
  created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_assets');