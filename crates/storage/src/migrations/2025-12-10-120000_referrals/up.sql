CREATE TABLE rewards_levels_types (
    id VARCHAR(32) PRIMARY KEY
);

CREATE TABLE usernames (
    username VARCHAR(64) PRIMARY KEY,
    address VARCHAR(256) NOT NULL UNIQUE,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('usernames');

CREATE TABLE rewards (
    username VARCHAR(64) PRIMARY KEY REFERENCES usernames(username) ON DELETE CASCADE ON UPDATE CASCADE,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    level VARCHAR(32) REFERENCES rewards_levels_types(id),
    points INT NOT NULL DEFAULT 0 CHECK (points >= 0),
    referrer_username VARCHAR(64) REFERENCES usernames(username) ON DELETE SET NULL ON UPDATE CASCADE,
    referral_count INT NOT NULL DEFAULT 0 CHECK (referral_count >= 0),
    device_id INTEGER NOT NULL REFERENCES devices(id),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('rewards');

CREATE TABLE rewards_risk_signals (
    id SERIAL PRIMARY KEY,
    fingerprint VARCHAR(64) NOT NULL,
    referrer_username VARCHAR(64) NOT NULL REFERENCES rewards(username) ON DELETE CASCADE ON UPDATE CASCADE,
    device_id INT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    device_platform VARCHAR(16) NOT NULL,
    device_os VARCHAR(32) NOT NULL,
    device_model VARCHAR(64) NOT NULL,
    device_locale VARCHAR(16) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    ip_country_code VARCHAR(2) NOT NULL,
    ip_usage_type VARCHAR(64) NOT NULL,
    ip_isp VARCHAR(128) NOT NULL,
    ip_abuse_score INT NOT NULL,
    risk_score INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE INDEX rewards_risk_signals_fingerprint_idx ON rewards_risk_signals(fingerprint);
CREATE INDEX rewards_risk_signals_referrer_username_idx ON rewards_risk_signals(referrer_username);
CREATE INDEX rewards_risk_signals_ip_address_idx ON rewards_risk_signals(ip_address);
CREATE INDEX rewards_risk_signals_device_id_idx ON rewards_risk_signals(device_id);

CREATE TABLE rewards_referrals (
    id SERIAL PRIMARY KEY,
    referrer_username VARCHAR(64) NOT NULL REFERENCES rewards(username) ON DELETE CASCADE ON UPDATE CASCADE,
    referred_username VARCHAR(64) NOT NULL REFERENCES rewards(username) ON DELETE CASCADE ON UPDATE CASCADE UNIQUE,
    referred_device_id INT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    risk_signal_id INT NOT NULL REFERENCES rewards_risk_signals(id),
    created_at timestamp NOT NULL default current_timestamp
);

CREATE INDEX rewards_referrals_referrer_idx ON rewards_referrals(referrer_username);
CREATE INDEX rewards_referrals_referred_device_id_idx ON rewards_referrals(referred_device_id);

CREATE TABLE rewards_events_types (
    id VARCHAR(64) PRIMARY KEY,
    points INT NOT NULL
);

CREATE TABLE rewards_events (
    id SERIAL PRIMARY KEY,
    username VARCHAR(64) NOT NULL REFERENCES usernames(username) ON DELETE CASCADE ON UPDATE CASCADE,
    event_type VARCHAR(64) NOT NULL REFERENCES rewards_events_types(id) ON DELETE CASCADE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE INDEX rewards_events_username_idx ON rewards_events(username);

SELECT diesel_manage_updated_at('rewards_events');
