CREATE TABLE fiat_assets (
  id SERIAL PRIMARY KEY,
  asset VARCHAR NOT NULL,
  provider VARCHAR NOT NULL,
  symbol VARCHAR NOT NULL,
  network VARCHAR NULL,
  UNIQUE(asset, provider, symbol)
);

INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('ethereum', 'MoonPay','eth',null);
INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('ethereum', 'Transak','ETH','ethereum');
INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('ethereum', 'Mercuryo','ETH',null);

INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', 'MoonPay','usdc',null);
INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', 'Transak','USDC','ethereum');
INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', 'Mercuryo','USDC',null);

INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('polygon', 'MoonPay','matic_polygon',null);
INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('polygon', 'Transak','MATIC','polygon');
INSERT INTO fiat_assets(asset, provider, symbol, network) VALUES ('polygon', 'Mercuryo','MATIC',null);
