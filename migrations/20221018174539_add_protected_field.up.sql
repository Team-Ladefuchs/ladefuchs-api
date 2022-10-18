-- Add up migration script here
ALTER TABLE if exists charge_price add column is_protected boolean NOT NULL DEFAULT false;
