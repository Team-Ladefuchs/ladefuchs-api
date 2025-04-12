-- Add up migration script here

create schema if not exists eco_movement;

create table if not exists eco_movement.location (
    id UUID primary key,
    value JSON not null
);
