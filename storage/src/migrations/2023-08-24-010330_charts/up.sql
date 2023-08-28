CREATE TABLE charts (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR NOT NULL,
    date TIMESTAMP NOT NULL,
    price float NOT NULL DEFAULT 0,
    market_cap float NOT NULL DEFAULT 0,
    volume float NOT NULL DEFAULT 0,
    UNIQUE(coin_id, date)
);

create index charts_date_trunc_minute on charts (date_trunc('minute', date));
create index charts_date_trunc_hour on charts (date_trunc('hour', date));
create index charts_date_trunc_day on charts (date_trunc('day', date));