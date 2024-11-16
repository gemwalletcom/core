CREATE TABLE fiat_rates (
  id SERIAL PRIMARY KEY,
  symbol VARCHAR NOT NULL,
  name VARCHAR NOT NULL,
  rate float NOT NULL DEFAULT 0,
  created_at timestamp NOT NULL default current_timestamp,
  updated_at timestamp NOT NULL default current_timestamp,
  UNIQUE(symbol)
);

SELECT diesel_manage_updated_at('fiat_rates');