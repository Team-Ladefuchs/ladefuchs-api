-- Add up migration script here
delete from msp where msp_id = 'c2aa5923-ea97-4923-bf41-e340451f3144';

delete from tarif where relationship_id = 'a67a4af3-aca2-4d9e-9b2c-91a0156e0950';

delete from vehicle_tarif where tarif_id = (select id from tarif where relationship_id = 'a67a4af3-aca2-4d9e-9b2c-91a0156e0950') and vehicle_id = (select id from vehicle where uuid = 'c1fd1277-5d77-416b-bb25-84bd21f57963');

