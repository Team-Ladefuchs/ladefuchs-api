-- Add down migration script here
ALTER TABLE tariff RENAME TO  tarif;

ALTER TABLE tariff_image RENAME TO tarif_image;

ALTER TABLE vehicle_tariff RENAME TO vehicle_tarif;