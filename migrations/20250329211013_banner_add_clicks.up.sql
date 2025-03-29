-- Add up migration script here
alter table link_banner add column impression int4 not null default 0;
