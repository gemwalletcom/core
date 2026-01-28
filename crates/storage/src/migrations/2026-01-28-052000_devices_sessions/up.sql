CREATE TABLE devices_sessions (
    id SERIAL PRIMARY KEY,
    device_id INT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    address VARCHAR(256) NOT NULL,
    signature VARCHAR(256) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE INDEX idx_devices_sessions_device_id ON devices_sessions (device_id);
CREATE INDEX idx_devices_sessions_address ON devices_sessions (address);
CREATE INDEX idx_devices_sessions_created_at ON devices_sessions (created_at DESC);
