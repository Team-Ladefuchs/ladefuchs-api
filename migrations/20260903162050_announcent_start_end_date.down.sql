-- Add down migration script here
alter table announcement drop column start;
alter table announcement drop column end;
