CREATE TYPE notification_type AS ENUM ('referralJoined', 'rewardsEnabled', 'rewardsCodeDisabled');

CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    wallet_id INTEGER NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    notification_type notification_type NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE INDEX notifications_wallet_id_idx ON notifications (wallet_id);
CREATE INDEX notifications_is_read_idx ON notifications (is_read);
CREATE INDEX notifications_created_at_idx ON notifications (created_at DESC);
