-- Add up migration script here
alter table if exists tariff_image RENAME to image;

alter table cpo add column if not exists image int constraint cpo_image_fk references image(id) on delete set null;

