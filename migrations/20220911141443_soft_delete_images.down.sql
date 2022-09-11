-- Add down migration script here
alter table tariff_image drop column soft_delete;
drop table if exists filter CASCADE;
