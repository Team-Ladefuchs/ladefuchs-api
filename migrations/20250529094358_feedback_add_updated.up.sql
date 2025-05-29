-- Add up migration script here
alter table feedback add column if not exists updated timestamptz not null default now();
