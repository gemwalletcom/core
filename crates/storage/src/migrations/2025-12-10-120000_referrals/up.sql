CREATE TABLE rewards_levels_types (
    id VARCHAR(32) PRIMARY KEY
);

CREATE TABLE usernames (
    username VARCHAR(64) PRIMARY KEY,
    address VARCHAR(256) NOT NULL UNIQUE,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_rewards_enabled BOOLEAN NOT NULL DEFAULT true,
    rewards_level VARCHAR(32) REFERENCES rewards_levels_types(id),
    points INT NOT NULL DEFAULT 0 CHECK (points >= 0),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('usernames');

CREATE TABLE rewards_referrals (
    id SERIAL PRIMARY KEY,
    referrer_username VARCHAR(64) NOT NULL REFERENCES usernames(username) ON DELETE CASCADE ON UPDATE CASCADE,
    referred_username VARCHAR(64) NOT NULL REFERENCES usernames(username) ON DELETE CASCADE ON UPDATE CASCADE UNIQUE,
    referred_device_id INT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE INDEX rewards_referrals_referrer_idx ON rewards_referrals(referrer_username);
CREATE INDEX rewards_referrals_referred_device_id_idx ON rewards_referrals(referred_device_id);

SELECT diesel_manage_updated_at('rewards_referrals');

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
