-- Add up migration script here
alter table affiliate_state add column link_banner_id int references link_banner(id) on delete set null
