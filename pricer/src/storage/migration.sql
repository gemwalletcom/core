CREATE TABLE IF NOT EXISTS charts
(
    coin_id            LowCardinality(String),
    price              Float64,
    created_at         DateTime64(0)
)
ENGINE = ReplacingMergeTree
    PARTITION BY (coin_id, toYYYYMMDD(created_at))
    ORDER BY (coin_id, created_at);