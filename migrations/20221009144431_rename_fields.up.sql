-- Add up migration script here
ALTER TABLE if exists msp drop column is_enabled;
ALTER TABLE if exists msp drop column msp_id;

ALTER TYPE plattformtype RENAME TO PlatformType;

ALTER TABLE affiliate_state RENAME COLUMN plattform TO platform;

ALTER TABLE affiliate_state RENAME TO affiliate_statistic;

ALTER table filter add column comment text;

ALTER table if exists tariff_image add  column is_ac_hoc bool default false;
