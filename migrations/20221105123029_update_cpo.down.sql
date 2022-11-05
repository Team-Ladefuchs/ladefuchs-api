-- Add down migration script here
alter table cpo add column expect_AC int4 not null default 3 check ( expect_AC >= 0);
alter table cpo add column expect_DC int4 not null default 3 check ( expect_DC >= 0);

alter table cpo drop column supported_types;

update cpo set expect_AC = 0
where network = '429bf694-699e-4156-8535-1554bb11f64e';

