-- Add down migration script here

ALTER TABLE tariff ADD COLUMN IF NOT EXISTS msp_id int;
SELECT replace_msp_id();
ALTER TABLE tariff DROP COLUMN IF EXISTS provider;
