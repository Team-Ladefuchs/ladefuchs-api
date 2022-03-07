-- -- Add up migration script here
insert into cpo(network, name, slug_name, extra)
Values('0b41a566-f637-4530-b743-7e73d1ed4fd9', 'allego', 'Allego', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}'); 

insert into cpo(network, name, slug_name, extra)
Values('8c21c2da-ee07-4930-bad2-9c0860dac62f', 'enbw', 'EnBW', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('4facbd5e-d179-4095-ad0e-5249b0c023c8', 'fastned', 'Fastned', 
'{"powerAC": 22,"powerDC": 100,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('429bf694-699e-4156-8535-1554bb11f64e', 'ionity', 'Ionity', 
'{"powerAC": 22,"powerDC": 350,"expectAC": 0,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('10c40598-5539-41e1-9f25-a120d13695a0', 'ladenetz', 'Ladenetz', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('3944b974-e319-4f1e-9241-321f859a3b07', 'innogy', 'Innogy', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('555294c8-26e1-4730-b8a8-f0bd6b0a75a1', 'beemobil', 'be emobil', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('24ef4b7e-7bf0-48d4-9c18-8edfca5908e0', 'be.energised', 'be.ENERGISED', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('e60ef10a-383c-4f35-a9f5-88e64323731b', 'e.on', 'E.ON', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('da798111-a97b-4262-b091-966330ae0d49', 'elli', 'Elli', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('0283232b-4d6b-459a-83ef-b2ae2bf083ed', 'comfortcharge', 'Comfort Charge', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('f31e02b3-5b28-4508-ae91-f4ccfbfbe0fb', 'ladeverbund+', 'Ladeverbund+', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

insert into cpo(network, name, slug_name, extra)
Values('fda62ff9-5ae7-4aca-8f50-2c224ae0c834', 'newmotion', 'New Motion', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');

-- old was ze-f1xha-ll3lu-j4h- ...
insert into cpo(network, name, slug_name, extra, is_enabled)
Values('d017dd34-3974-4bee-a10e-abca046a9c48', 'marliquenergy+', 'marLIQUEnergy', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}', false);