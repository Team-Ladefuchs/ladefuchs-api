-- Add down migration script here
ALTER TABLE if exists charge_price drop column is_protected;
