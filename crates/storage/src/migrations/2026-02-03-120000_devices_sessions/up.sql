CREATE TABLE devices_sessions (
    id SERIAL PRIMARY KEY,
    device_id INT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE INDEX devices_sessions_device_id_idx ON devices_sessions (device_id);
CREATE INDEX devices_sessions_created_at_idx ON devices_sessions (created_at);
