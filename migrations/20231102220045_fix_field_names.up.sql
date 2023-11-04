-- Add up migration script here
alter table if exists charge_price rename column cpo_id to operator_id;
alter table if exists charge_price rename column blockingfee to blocking_fee;
alter table if exists operator rename column is_enabled to standard;
alter table if exists operator drop column if exists hide;
alter table if exists tariff add column hide bool not null default false;
