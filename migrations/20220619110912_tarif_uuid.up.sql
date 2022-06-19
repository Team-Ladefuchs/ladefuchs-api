-- Add up migration script here

alter table tarif add pub_tarif_id uuid default gen_random_uuid() not null