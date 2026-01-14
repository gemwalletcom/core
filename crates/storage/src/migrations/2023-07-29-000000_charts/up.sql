CREATE TABLE IF NOT EXISTS charts (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS charts_hourly (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS charts_daily (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, created_at)
);

-- indexes
CREATE INDEX IF NOT EXISTS idx_charts_created_at ON charts (created_at);
CREATE INDEX IF NOT EXISTS idx_charts_hourly_created_at ON charts_hourly (created_at);
CREATE INDEX IF NOT EXISTS idx_charts_daily_created_at ON charts_daily (created_at);

-- functions
CREATE OR REPLACE FUNCTION aggregate_hourly_charts() RETURNS VOID AS $$
BEGIN
    INSERT INTO charts_hourly (coin_id, created_at, price)
    SELECT
        charts.coin_id,
        DATE_TRUNC('hour', charts.created_at),
        AVG(charts.price)
    FROM charts
    WHERE charts.created_at >= DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour'
    GROUP BY charts.coin_id, DATE_TRUNC('hour', charts.created_at)
    ON CONFLICT (coin_id, created_at) DO UPDATE SET price = EXCLUDED.price;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION aggregate_daily_charts() RETURNS VOID AS $$
BEGIN
    INSERT INTO charts_daily (coin_id, created_at, price)
    SELECT
        charts_hourly.coin_id,
        DATE_TRUNC('day', charts_hourly.created_at),
        AVG(charts_hourly.price)
    FROM charts_hourly
    WHERE charts_hourly.created_at >= DATE_TRUNC('day', NOW()) - INTERVAL '1 day'
    GROUP BY charts_hourly.coin_id, DATE_TRUNC('day', charts_hourly.created_at)
    ON CONFLICT (coin_id, created_at) DO UPDATE SET price = EXCLUDED.price;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION cleanup_all_charts_data() RETURNS VOID AS $$
BEGIN
    DELETE FROM charts WHERE created_at < NOW() - INTERVAL '8 days';
    DELETE FROM charts_hourly WHERE created_at < NOW() - INTERVAL '31 days';
    DELETE FROM charts_daily WHERE created_at < NOW() - INTERVAL '365 days';
END;
$$ LANGUAGE plpgsql;

ALTER TABLE charts SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_vacuum_threshold = 1000);
ALTER TABLE charts_hourly SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_vacuum_threshold = 500);
ALTER TABLE charts_daily SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_vacuum_threshold = 500);