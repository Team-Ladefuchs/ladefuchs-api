-- Add down migration script here
drop table if exists charge_price CASCADE;
drop table if exists tarif CASCADE;
drop table if exists vehicle CASCADE;
drop table if exists cpo CASCADE;
drop table if exists msp CASCADE;
drop type if exists ChargeType;
drop type if exists VehicleType;