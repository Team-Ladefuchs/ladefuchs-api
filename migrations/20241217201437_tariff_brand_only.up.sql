-- Add up migration script here
alter table tariff add column if not exists brand_only bool not null default false;
alter table tariff drop column if exists override_standard;
