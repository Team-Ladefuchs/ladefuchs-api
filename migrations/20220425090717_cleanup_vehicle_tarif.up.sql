-- Add up migration script here
delete from charge_price;
delete from vehicle_tarif;

-- rm empty car
delete from vehicle where uuid = 'c2906db7-6efd-474f-bba5-7e128aa0477f';
alter table vehicle drop column if exists  vehicle_type;
drop type if exists vehicletype;
update vehicle set is_enabled = true;


-- enbew msp
insert into msp(msp_id, pub_msp_id, name, is_enabled, legacy_id) values
          ('c2aa5923-ea97-4923-bf41-e340451f3144',   'de804ad0-14d6-492c-81e6-80cec37a05eb', 'EnBW', true, 'enbw') 
    on conflict do nothing;

-- Hyundai Sondertarif
insert into tarif(relationship_id, msp_id, slug_name) VALUES
    ('a67a4af3-aca2-4d9e-9b2c-91a0156e0950', 
    (select id from msp where name = 'EnBW' and legacy_id = 'enbw'), 
            'Hyundai Sondertarif') 
    on conflict do nothing;

insert into vehicle_tarif(tarif_id, vehicle_id) VALUES (
    (select id from tarif where relationship_id = 'a67a4af3-aca2-4d9e-9b2c-91a0156e0950'),
    (select id from vehicle where uuid = 'c1fd1277-5d77-416b-bb25-84bd21f57963') 
) on conflict do nothing;
