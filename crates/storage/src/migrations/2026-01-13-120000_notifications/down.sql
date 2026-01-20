DROP INDEX IF EXISTS notifications_created_at_idx;
DROP INDEX IF EXISTS notifications_is_read_idx;
DROP INDEX IF EXISTS notifications_wallet_id_idx;
DROP TABLE IF EXISTS notifications;
DROP TYPE IF EXISTS notification_type;
