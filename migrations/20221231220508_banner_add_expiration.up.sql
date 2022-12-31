-- Add up migration script here
alter table link_banner add column expiration timestamptz NOT NULL DEFAULT date '2030-12-31 22:13:06.255001 +00:00';
