CREATE TYPE reward_status AS ENUM ('unverified', 'pending', 'verified', 'trusted', 'disabled');
CREATE TYPE reward_event_type AS ENUM ('createUsername', 'invitePending', 'inviteNew', 'inviteExisting', 'joined', 'disabled');
CREATE TYPE username_status AS ENUM ('unverified', 'verified');

CREATE TABLE usernames (
    username VARCHAR(64) PRIMARY KEY,
    address VARCHAR(256) NOT NULL UNIQUE,
    status username_status NOT NULL DEFAULT 'unverified',
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('usernames');

CREATE TABLE rewards (
    username VARCHAR(64) PRIMARY KEY REFERENCES usernames(username) ON DELETE CASCADE ON UPDATE CASCADE,
    status reward_status NOT NULL,
    level VARCHAR(32),
    points INT NOT NULL DEFAULT 0 CHECK (points >= 0),
    referrer_username VARCHAR(64) REFERENCES usernames(username) ON DELETE SET NULL ON UPDATE CASCADE,
    referral_count INT NOT NULL DEFAULT 0 CHECK (referral_count >= 0),
    device_id INTEGER NOT NULL REFERENCES devices(id),
    is_swap_complete BOOLEAN NOT NULL DEFAULT false,
    comment VARCHAR(512),
    disable_reason VARCHAR(256),
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
    device_platform_store VARCHAR(32) NOT NULL,
    device_os VARCHAR(32) NOT NULL,
    device_model VARCHAR(64) NOT NULL,
    device_locale VARCHAR(16) NOT NULL,
    device_currency VARCHAR(8) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    ip_country_code VARCHAR(2) NOT NULL,
    ip_usage_type VARCHAR(64) NOT NULL,
    ip_isp VARCHAR(128) NOT NULL,
    ip_abuse_score INT NOT NULL,
    risk_score INT NOT NULL,
    metadata JSONB,
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
    verified_at TIMESTAMP,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE INDEX rewards_referrals_referrer_idx ON rewards_referrals(referrer_username);
CREATE INDEX rewards_referrals_referred_device_id_idx ON rewards_referrals(referred_device_id);

SELECT diesel_manage_updated_at('rewards_referrals');

CREATE TABLE rewards_events (
    id SERIAL PRIMARY KEY,
    username VARCHAR(64) NOT NULL REFERENCES usernames(username) ON DELETE CASCADE ON UPDATE CASCADE,
    event_type reward_event_type NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE INDEX rewards_events_username_idx ON rewards_events(username);

SELECT diesel_manage_updated_at('rewards_events');

CREATE TABLE rewards_referral_attempts (
    id SERIAL PRIMARY KEY,
    referrer_username VARCHAR(64) NOT NULL REFERENCES rewards(username) ON UPDATE CASCADE ON DELETE CASCADE,
    referred_address VARCHAR(256) NOT NULL,
    device_id INTEGER NOT NULL REFERENCES devices(id),
    risk_signal_id INT NULL REFERENCES rewards_risk_signals(id) ON DELETE SET NULL,
    reason VARCHAR(256) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_rewards_referral_attempts_referrer_username ON rewards_referral_attempts(referrer_username);
CREATE INDEX idx_rewards_referral_attempts_created_at ON rewards_referral_attempts(created_at);
