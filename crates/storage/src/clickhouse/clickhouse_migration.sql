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
