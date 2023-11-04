-- Add down migration script here
alter table if exists charge_price rename column operator_id to cpo_id;
alter table if exists charge_price rename column blocking_fee to blockingfee;
alter table if exists operator add column hide bool not null default false;
alter table if exists tariff drop column if exists hide;
alter table if exists operator rename column standard to is_enabled;
