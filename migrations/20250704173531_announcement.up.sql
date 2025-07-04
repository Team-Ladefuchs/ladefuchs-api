-- Add up migration script here
create table if not exists announcement (
    id UUID primary key default gen_random_uuid(),
    value JSON not null
);
