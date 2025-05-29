-- Add down migration script here
ALTER TABLE feedback DROP COLUMN IF EXISTS updated;
