-- Add down migration script here
alter table tariff drop column if exists is_enabled	
