-- Add up migration script here
alter table announcement add start_at timestamptz;
alter table announcement add end_at timestamptz;
