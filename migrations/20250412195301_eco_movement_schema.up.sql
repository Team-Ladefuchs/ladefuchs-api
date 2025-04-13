-- Add up migration script here

create schema if not exists eco_movement;

create table if not exists eco_movement.location (
    id UUID primary key,
    value JSON not null
);

create table if not exists eco_movement.connector_price (
    location_id UUID,
    pricing_id TEXT not null check (pricing_id <> ''),
    primary key (location_id, pricing_id),
    evse_uid TEXT not null check (evse_uid <> ''),
    connector_id TEXT not null
);

create table if not exists eco_movement.price (
    id TEXT primary key,
    value JSON not null
);

create table if not exists eco_movement.tariff (
    id TEXT primary key,
    value JSON not null
);
