CREATE TABLE chains (
    id VARCHAR(32) PRIMARY KEY NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('chains');

INSERT INTO "chains" ("id") VALUES ('ethereum');
INSERT INTO "chains" ("id") VALUES ('cosmos');
INSERT INTO "chains" ("id") VALUES ('tron');
INSERT INTO "chains" ("id") VALUES ('smartchain');
INSERT INTO "chains" ("id") VALUES ('binance');
INSERT INTO "chains" ("id") VALUES ('osmosis');
INSERT INTO "chains" ("id") VALUES ('solana');
INSERT INTO "chains" ("id") VALUES ('arbitrum');
INSERT INTO "chains" ("id") VALUES ('polygon');
INSERT INTO "chains" ("id") VALUES ('bitcoin');
