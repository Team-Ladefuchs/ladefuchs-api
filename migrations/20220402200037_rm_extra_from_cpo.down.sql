-- Add down migration script here
drop procedure if exists change_cpo;

alter table cpo add extra json NOT NULL;

alter table cpo drop column power_AC;
alter table cpo drop column power_DC;
alter table cpo drop column expect_AC;
alter table cpo drop column expect_DC;