-- Add down migration script here
alter table if exists charge_price rename column cpo_id to operator_id;
alter table if exists charge_price rename column blockingfee to blockingfee;
alter table if exists operator drop column if exists hide;
alter table if exists tariff drop column if exists hide;
