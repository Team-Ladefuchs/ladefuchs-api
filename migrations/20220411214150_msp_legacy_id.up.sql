-- Add up migration script here
ALTER table msp add column legacy_id text default '' not null;