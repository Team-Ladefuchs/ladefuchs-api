-- Add up migration script here
ALTER TABLE tariff_image RENAME COLUMN is_ac_hoc TO is_ad_hoc;
ALTER TABLE charge_price add COLUMN if not exists blockingFee double precision NOT NULL DEFAULT 0;
