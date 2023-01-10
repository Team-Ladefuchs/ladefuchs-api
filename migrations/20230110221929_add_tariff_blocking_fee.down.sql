-- Add down migration script here
ALTER TABLE tariff_image RENAME COLUMN is_ad_hoc TO is_ac_hoc;
ALTER TABLE charge_price drop COLUMN if exists blockingFee;
