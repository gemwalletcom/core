CREATE TABLE subscriptions (
    id SERIAL PRIMARY KEY,
    device_id INTEGER NOT NULL,
    chain VARCHAR NOT NULL,
    address VARCHAR(256) NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(device_id, chain, address)
);

ALTER TABLE subscriptions
      ADD CONSTRAINT fk_device_id_subscriptions FOREIGN KEY (device_id) 
          REFERENCES devices (id);

SELECT diesel_manage_updated_at('subscriptions');