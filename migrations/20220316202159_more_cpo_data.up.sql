-- Add up migration script here
insert into cpo(network, name, slug_name, extra)
Values('2ac29d9c-2ce9-4d8e-a8b8-297aedf4ea2e', 'aral', 'Aral Pulse', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('606a4593-576a-4dd3-a43b-e890063314d0', 'ewe', 'EWE/SWB', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('83af72cc-65ee-4cca-922a-e5031024c32f', 'ladefoxx', 'Ladefoxx', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 1,"expectDC": 1}');

insert into cpo(network, name, slug_name, extra, is_enabled)
Values('3ad0f93f-8f41-476d-b214-3b892d6399e0', 'gpjoule', 'GP Joule"', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 0}', false);

update cpo
set is_enabled = false
where 
    network = 'da798111-a97b-4262-b091-966330ae0d49' or
    network = '429bf694-699e-4156-8535-1554bb11f64e' or
    network = 'fda62ff9-5ae7-4aca-8f50-2c224ae0c834' or
    network = '555294c8-26e1-4730-b8a8-f0bd6b0a75a1';