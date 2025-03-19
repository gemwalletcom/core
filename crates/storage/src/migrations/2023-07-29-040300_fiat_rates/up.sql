CREATE TABLE fiat_rates (
  id VARCHAR(8) NOT NULL PRIMARY KEY,
  name VARCHAR NOT NULL,
  rate float NOT NULL DEFAULT 0,
  created_at timestamp NOT NULL default current_timestamp,
  updated_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_rates');