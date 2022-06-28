-- Add up migration script here
create table IF NOT EXISTS admin_user (
    id serial primary key,
    username text unique not null check ( username <> ''),
    password_hash text not null check ( password_hash <> '' ),
    created timestamptz not null default CURRENT_TIMESTAMP
);