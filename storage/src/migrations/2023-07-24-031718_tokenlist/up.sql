CREATE TABLE tokenlists (
  id SERIAL PRIMARY KEY,
  chain VARCHAR NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
  url VARCHAR NOT NULL,
  version INTEGER NOT NULL,
  UNIQUE(chain, url)
);

INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('ethereum', 'https://assets.gemwallet.com/tokenlists/ethereum.json', 3);
INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('cosmos', 'https://assets.gemwallet.com/tokenlists/cosmos.json', 1);
INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('tron', 'https://assets.gemwallet.com/tokenlists/tron.json', 1);
INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('smartchain', 'https://assets.gemwallet.com/tokenlists/smartchain.json', 2);
INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('binance', 'https://assets.gemwallet.com/tokenlists/binance.json', 1);
INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('osmosis', 'https://assets.gemwallet.com/tokenlists/osmosis.json', 1);
INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('solana', 'https://assets.gemwallet.com/tokenlists/solana.json', 1);
INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('arbitrum', 'https://assets.gemwallet.com/tokenlists/arbitrum.json', 1);
INSERT INTO "tokenlists" ("chain", "url", "version") VALUES ('polygon', 'https://assets.gemwallet.com/tokenlists/polygon.json', 1);
