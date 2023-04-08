-- Add down migration script here
alter table charge_price
    drop constraint if exists charge_price_cpo_id_fkey;

alter table charge_price
    add constraint charge_price_cpo_id_fkey
    foreign key (cpo_id) references cpo;
