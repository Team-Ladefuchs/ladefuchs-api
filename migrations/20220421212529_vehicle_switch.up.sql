-- Add up migration script here
alter table vehicle add column is_enabled boolean not null default false;

-- enabled default car
update vehicle set is_enabled = true where uuid = 'c2906db7-6efd-474f-bba5-7e128aa0477f';

-- enable newmotion
update cpo set is_enabled = true where network = 'fda62ff9-5ae7-4aca-8f50-2c224ae0c834';