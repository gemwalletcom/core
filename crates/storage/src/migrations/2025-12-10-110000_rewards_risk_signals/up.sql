CREATE TABLE rewards_risk_signals (
    id SERIAL PRIMARY KEY,
    fingerprint VARCHAR(64) NOT NULL,
    username VARCHAR(64) NOT NULL REFERENCES rewards(username) ON DELETE CASCADE ON UPDATE CASCADE,
    device_id INT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Device signals
    device_platform VARCHAR(16) NOT NULL,
    device_os VARCHAR(32) NOT NULL,
    device_model VARCHAR(64) NOT NULL,
    device_locale VARCHAR(16) NOT NULL,

    -- IP signals
    ip_address VARCHAR(45) NOT NULL,
    ip_country_code VARCHAR(2) NOT NULL,
    ip_usage_type VARCHAR(64) NOT NULL,
    ip_isp VARCHAR(128) NOT NULL,
    ip_abuse_score INT NOT NULL,
    risk_score INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE INDEX rewards_risk_signals_fingerprint_idx ON rewards_risk_signals(fingerprint);
CREATE INDEX rewards_risk_signals_username_idx ON rewards_risk_signals(username);
CREATE INDEX rewards_risk_signals_ip_address_idx ON rewards_risk_signals(ip_address);
CREATE INDEX rewards_risk_signals_device_id_idx ON rewards_risk_signals(device_id);
