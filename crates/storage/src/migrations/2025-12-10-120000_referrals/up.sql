CREATE TABLE referrals (
    id SERIAL PRIMARY KEY,
    address VARCHAR(256) NOT NULL UNIQUE,
    code VARCHAR(64) UNIQUE,
    used_referral_code VARCHAR(64),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('referrals');

CREATE TABLE referrals_uses (
    id SERIAL PRIMARY KEY,
    referrer_address VARCHAR(256) NOT NULL REFERENCES referrals (address) ON DELETE CASCADE,
    referred_address VARCHAR(256) NOT NULL REFERENCES referrals (address) ON DELETE CASCADE UNIQUE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE INDEX referrals_uses_referrer_idx ON referrals_uses (referrer_address);

SELECT diesel_manage_updated_at('referrals_uses');

CREATE TABLE referrals_events_types (
    id VARCHAR(64) PRIMARY KEY,
    points INT NOT NULL
);

CREATE TABLE referrals_events (
    id SERIAL PRIMARY KEY,
    address VARCHAR(256) NOT NULL REFERENCES referrals (address) ON DELETE CASCADE,
    event_type VARCHAR(64) NOT NULL REFERENCES referrals_events_types (id) ON DELETE CASCADE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE INDEX referrals_events_address_idx ON referrals_events (address);

SELECT diesel_manage_updated_at('referrals_events');
