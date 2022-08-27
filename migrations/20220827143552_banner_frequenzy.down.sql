-- Add down migration script here
ALTER TABLE if exists link_banner add column high_priority bool not null default false;
ALTER TABLE if exists link_banner drop column frequency;
ALTER TABLE if exists cpo drop column hide;

