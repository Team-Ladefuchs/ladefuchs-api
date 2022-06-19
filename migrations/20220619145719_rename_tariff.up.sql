-- Add up migration script here
ALTER TABLE if exists  tarif RENAME TO tariff;

ALTER TABLE if exists  tarif_image RENAME TO tariff_image;

ALTER TABLE if exists  vehicle_tarif RENAME TO vehicle_tariff;

ALTER TABLE if exists vehicle_tariff RENAME COLUMN tarif_id TO tariff_id;

ALTER TABLE if exists tariff RENAME COLUMN pub_tarif_id TO pub_tariff_id;