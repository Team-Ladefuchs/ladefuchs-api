-- Add up migration script here
drop table if exists tarif_image cascade;
drop procedure if exists internal_tarif_name;


alter table tarif drop column image;
alter table tarif drop column url;
alter table tarif drop internal_name;

