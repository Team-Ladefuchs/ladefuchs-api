-- Add up migration script here
drop table if exists vehicle_tariff cascade;
drop table if exists vehicle cascade;
drop type if exists VEHICLETYPE;
drop table if exists filter;
alter table if exists tariff drop column if exists alternative_operator_name;
