-- Add down migration script here
alter table vehicle drop column is_enabled;

-- disbale newmotion
update cpo set is_enabled = false where network = 'f548d92a-e9e6-4ec8-8b63-4f6fb9b86711';