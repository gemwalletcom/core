CREATE TABLE IF NOT EXISTS price_alerts (
    id SERIAL PRIMARY KEY,
    identifier varchar(512) NOT NULL,

    device_id            INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    asset_id             VARCHAR(128) NOT NULL REFERENCES assets (id)  ON DELETE CASCADE,
    currency             VARCHAR(128) NOT NULL REFERENCES fiat_rates (id)  ON DELETE CASCADE,
    price_direction      VARCHAR(16),
    price                float,
    price_percent_change float,
    last_notified_at     timestamp,

    updated_at           timestamp NOT NULL default current_timestamp,
    created_at           timestamp NOT NULL default current_timestamp,

    UNIQUE (device_id, identifier)
);

SELECT diesel_manage_updated_at('price_alerts');

CREATE INDEX price_alerts_asset_id_idx ON price_alerts (asset_id);
CREATE INDEX price_alerts_last_notified_at_idx ON price_alerts (last_notified_at);