DROP TRIGGER IF EXISTS affiliate_statistic_daily_trigger ON affiliate_statistic;
DROP FUNCTION IF EXISTS trg_affiliate_statistic_daily();
DROP TRIGGER IF EXISTS impression_banner_daily_trigger ON impression_banner;
DROP FUNCTION IF EXISTS trg_impression_banner_daily();
DROP TABLE IF EXISTS affiliate_statistic_daily;
DROP TABLE IF EXISTS impression_banner_daily;
