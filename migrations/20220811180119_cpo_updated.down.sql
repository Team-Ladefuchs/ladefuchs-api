-- Add down migration script here
ALTER TABLE if exists charge_price RENAME COLUMN tariff_id TO tarif_id
ALTER TABLE if exists cpo drop column updated;
