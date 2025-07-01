-- Drop functions
DROP FUNCTION IF EXISTS aggregate_hourly_charts();
DROP FUNCTION IF EXISTS aggregate_daily_charts();
DROP FUNCTION IF EXISTS cleanup_all_charts_data();

-- Drop tables
DROP TABLE IF EXISTS charts_daily;
DROP TABLE IF EXISTS charts_hourly;
DROP TABLE IF EXISTS charts;
