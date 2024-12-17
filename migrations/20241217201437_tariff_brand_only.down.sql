-- Add down migration script here

alter table tariff drop column if exists brand_only;
alter table tariff add column if not exists override_standard bool not null default false;
