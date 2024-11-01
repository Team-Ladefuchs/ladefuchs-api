-- Add up migration script here
ALTER TABLE tariff ADD IF NOT EXISTS ad_hoc BOOLEAN DEFAULT false NOT NULL
