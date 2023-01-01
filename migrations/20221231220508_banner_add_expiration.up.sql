-- Add up migration script here
alter table link_banner add column if not exists expiration timestamptz NOT NULL DEFAULT date '2030-12-31 22:13:06.255001 +00:00';
alter table link_banner add column if not exists starts timestamptz NOT NULL DEFAULT now();
