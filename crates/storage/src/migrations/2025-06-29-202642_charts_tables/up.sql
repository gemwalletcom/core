-- Create the main charts table for raw data
CREATE TABLE charts (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    ts TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, ts)
);

CREATE TABLE charts_hourly_avg (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    ts TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, ts)
);

CREATE TABLE charts_daily_avg (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    ts TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, ts)
);

-- Add index for performance on the raw charts table
CREATE INDEX idx_charts_coin_id_ts ON charts (coin_id, ts DESC);
CREATE INDEX idx_charts_daily_coin_id_ts ON charts_daily_avg (coin_id, ts DESC);
CREATE INDEX idx_charts_hourly_coin_id_ts ON charts_hourly_avg (coin_id, ts DESC);

-- Function to aggregate data for the previous hour from raw charts table
CREATE OR REPLACE FUNCTION aggregate_hourly_charts() RETURNS void AS $$
BEGIN
    INSERT INTO charts_hourly_avg (coin_id, price, ts)
    SELECT
        coin_id,
        AVG(price),
        date_trunc('hour', ts)
    FROM charts
    WHERE ts >= date_trunc('hour', NOW() - INTERVAL '1 hour')
      AND ts < date_trunc('hour', NOW())
    GROUP BY coin_id, date_trunc('hour', ts)
    ON CONFLICT (coin_id, ts) DO UPDATE SET price = EXCLUDED.price;
END;
$$
LANGUAGE plpgsql;

-- Function to aggregate data for the previous day from hourly aggregates
CREATE OR REPLACE FUNCTION aggregate_daily_charts() RETURNS void AS $$
BEGIN
    INSERT INTO charts_daily_avg (coin_id, price, ts)
    SELECT
        coin_id,
        AVG(price),
        date_trunc('day', ts)
    FROM
        charts_hourly_avg
    WHERE
        ts >= date_trunc('day', now() - INTERVAL '1 day') AND ts < date_trunc('day', now())
    GROUP BY
        coin_id, date_trunc('day', ts)
    ON CONFLICT (coin_id, ts) DO UPDATE SET price = EXCLUDED.price;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up old charts data (e.g., older than 30 days)
CREATE OR REPLACE FUNCTION cleanup_old_charts_data() RETURNS void AS $$
BEGIN
    DELETE FROM charts WHERE ts < NOW() - INTERVAL '30 days';
    DELETE FROM charts_hourly_avg WHERE ts < NOW() - INTERVAL '30 days';
    DELETE FROM charts_daily_avg WHERE ts < NOW() - INTERVAL '30 days';
END;
$$
LANGUAGE plpgsql;

-- Function to insert charts from prices table every minute
CREATE OR REPLACE FUNCTION aggregate_charts() RETURNS void AS $$
BEGIN
    INSERT INTO charts (coin_id, price, ts)
    SELECT
        p.id AS coin_id,
        p.price,
        date_trunc('minute', NOW()) AS ts
    FROM prices p
    ON CONFLICT (coin_id, ts) DO NOTHING;
END;
$$
LANGUAGE plpgsql;

