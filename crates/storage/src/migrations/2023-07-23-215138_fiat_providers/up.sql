CREATE TABLE fiat_providers (
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    name VARCHAR(32) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

INSERT INTO fiat_providers(id, name, enabled) VALUES ('moonpay', 'MoonPay', true);
INSERT INTO fiat_providers(id, name, enabled) VALUES ('ramp', 'Ramp', true);
INSERT INTO fiat_providers(id, name, enabled) VALUES ('mercuryo', 'Mercuryo', true);
INSERT INTO fiat_providers(id, name, enabled) VALUES ('transak', 'Transak', true);

SELECT diesel_manage_updated_at('fiat_providers');