-- Add down migration script here
drop table if exists link_banner CASCADE;
drop table if exists affiliate_state CASCADE;
drop table if exists link CASCADE;

drop type if exists PlattformType;
