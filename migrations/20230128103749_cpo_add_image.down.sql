-- Add down migration script here
alter table if exists image RENAME to tariff_image;

ALTER TABLE cpo DROP CONSTRAINT IF EXISTS cpo_image_fk;

ALTER TABLE cpo drop column if exists image;

