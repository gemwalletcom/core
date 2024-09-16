CREATE TABLE IF NOT EXISTS price_alerts (
    id SERIAL  PRIMARY KEY,

    device_id            INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    asset_id             VARCHAR(128) NOT NULL REFERENCES assets (id)  ON DELETE CASCADE,
    price                float,
    last_notified_at     timestamp,

    updated_at           timestamp NOT NULL default current_timestamp,
    created_at           timestamp NOT NULL default current_timestamp,

    UNIQUE NULLS NOT DISTINCT(device_id, asset_id, price)
);

SELECT diesel_manage_updated_at('price_alerts');

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'devices'
        AND column_name = 'is_price_alerts_enabled'
    ) THEN
        ALTER TABLE devices
        ADD COLUMN is_price_alerts_enabled boolean NOT NULL DEFAULT false;
    END IF;
END $$;
