-- Add up migration script here
alter table charge_price
    drop constraint charge_price_cpo_id_fkey;

alter table charge_price
    add foreign key (cpo_id) references cpo
        on delete cascade;
