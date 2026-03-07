CREATE TYPE push_notification_type AS ENUM ('test', 'transaction', 'asset', 'priceAlert', 'buyAsset', 'swapAsset', 'support', 'rewards', 'stake');

CREATE TABLE devices_notifications (
    id SERIAL PRIMARY KEY,
    device_id INT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    notification_type push_notification_type NOT NULL,
    error VARCHAR(512),
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE INDEX devices_notifications_device_id_idx ON devices_notifications (device_id);
CREATE INDEX devices_notifications_created_at_idx ON devices_notifications (created_at DESC);
