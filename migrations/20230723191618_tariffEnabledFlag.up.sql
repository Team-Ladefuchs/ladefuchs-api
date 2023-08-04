-- Add up migration script here
alter table tariff add column if not exists is_enabled bool NOT NULL default true
