CREATE TABLE rewards_referral_attempts (
    id SERIAL PRIMARY KEY,
    referrer_username VARCHAR(64) NOT NULL REFERENCES rewards(username) ON UPDATE CASCADE ON DELETE CASCADE,
    referred_address VARCHAR(256) NOT NULL,
    country_code VARCHAR(2) NOT NULL,
    device_id INTEGER NOT NULL REFERENCES devices(id),
    referred_ip_address VARCHAR(45) NOT NULL,
    reason VARCHAR(256) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_rewards_referral_attempts_referrer_username ON rewards_referral_attempts(referrer_username);
CREATE INDEX idx_rewards_referral_attempts_created_at ON rewards_referral_attempts(created_at);
