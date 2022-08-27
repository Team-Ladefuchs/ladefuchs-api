-- Add up migration script here
ALTER TABLE if exists link_banner drop column high_priority;
ALTER TABLE if exists link_banner add column frequency smallint check ( frequency between 1 and 10) not null default 1;
ALTER TABLE if exists cpo add column hide bool default false;
