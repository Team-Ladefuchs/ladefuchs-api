-- Add up migration script here
update tarif
set alternative_operator_name = 'adac'
where relationship_id = '36c2017a-097a-4072-9a58-6ef904b6173d';

update tarif
set alternative_operator_name = 'elli'
where relationship_id = '91447011-4543-4509-b7b9-2aa90aa3ba63';

update tarif
set alternative_operator_name = 'ewp_echtmobil'
where relationship_id = '7a42726c-5a39-4a30-9299-d3945210149e';

update tarif
set alternative_operator_name = 'dew21'
where relationship_id = '1ea846df-919d-4786-9f7e-d26ec7d54650';

update tarif
set alternative_operator_name = 'charge4go'
where relationship_id = '24803992-59f3-4589-b149-5249d9e5ade0';

update tarif
set alternative_operator_name = 'stadtwerke_munchen_swm_'
where relationship_id = '0f0b6d38-14fb-4ffd-ac71-e83d5a368278';

update tarif
set alternative_operator_name = 'stadtwerke_kiel'
where relationship_id = 'a15898c9-8aea-4d06-a734-50342a6aa980';

update tarif
set alternative_operator_name = 'stadtwerke_kiel'
where relationship_id = 'a15898c9-8aea-4d06-a734-50342a6aa980';

update tarif
set internal_name = 'aral'
where relationship_id = 'a9227ef4-0c2c-447f-82c2-7db911c15655';

-- disable ladefoxx
update cpo
set is_enabled = false
where network = '83af72cc-65ee-4cca-922a-e5031024c32f';

-- rename aral -> aralpulse
update cpo
set name = 'aralpulse'
where network = '2ac29d9c-2ce9-4d8e-a8b8-297aedf4ea2e';


