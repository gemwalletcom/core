CREATE TABLE devices_sessions (
    id SERIAL PRIMARY KEY,
    device_id INT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    wallet_id INT NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    nonce VARCHAR(256) NOT NULL,
    signature VARCHAR(256) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE INDEX idx_devices_sessions_device_id ON devices_sessions (device_id);
CREATE INDEX idx_devices_sessions_wallet_id ON devices_sessions (wallet_id);
CREATE INDEX idx_devices_sessions_created_at ON devices_sessions (created_at DESC);
