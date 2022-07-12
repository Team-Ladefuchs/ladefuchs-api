-- Add down migration script here
drop table if exists msp_cpo cascade;
alter table charge_price drop constraint charge_price_tariff_id_fkey;
alter table charge_price add constraint charge_price_tarif_id_fkey foreign key(tarif_id) references tariff(id) on delete cascade;
