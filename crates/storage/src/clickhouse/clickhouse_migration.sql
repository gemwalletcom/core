CREATE TABLE IF NOT EXISTS charts
(
    coin_id            LowCardinality(String),
    price              Float32 CODEC(ZSTD(1)),
    ts                 DateTime CODEC(DoubleDelta, ZSTD(1))
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(ts)
ORDER BY (coin_id, toStartOfDay(ts), toStartOfHour(ts), toStartOfFifteenMinutes(ts), toStartOfFiveMinute(ts))
TTL
ts + INTERVAL 1 HOUR GROUP BY coin_id, toStartOfDay(ts), toStartOfHour(ts), toStartOfFifteenMinutes(ts), toStartOfFiveMinute(ts) SET price = avg(price),
ts + INTERVAL 1 DAY GROUP BY coin_id, toStartOfDay(ts), toStartOfHour(ts), toStartOfFifteenMinutes(ts) SET price = avg(price),
ts + INTERVAL 1 WEEK GROUP BY coin_id, toStartOfDay(ts), toStartOfHour(ts) SET price = avg(price),
ts + INTERVAL 1 MONTH GROUP BY coin_id, toStartOfDay(ts) SET price = avg(price);


CREATE TABLE IF NOT EXISTS positions
(
    store            LowCardinality(String),
    app            	 LowCardinality(String),
	keyword          LowCardinality(String),
    country          LowCardinality(String),
    position         UInt32 CODEC(ZSTD(1)),
    ts               Date CODEC(DoubleDelta, ZSTD(1))
)
ENGINE = MergeTree
ORDER BY (store, app, keyword, country, ts);

CREATE TABLE IF NOT EXISTS appstore_information
(
    store            LowCardinality(String),
    app              LowCardinality(String),
    country          LowCardinality(String),
    ratings          Float32 CODEC(ZSTD(1)),
    average_rating   Float32 CODEC(ZSTD(1)),
    ts               Date CODEC(DoubleDelta, ZSTD(1))
)
ENGINE = MergeTree
ORDER BY (store, app, country, ts);
