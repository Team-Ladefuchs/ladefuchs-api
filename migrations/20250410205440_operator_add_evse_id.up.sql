-- Add up migration script here
ALTER TABLE operator ADD COLUMN evse_id TEXT [] NOT NULL DEFAULT ARRAY[]::TEXT [];
