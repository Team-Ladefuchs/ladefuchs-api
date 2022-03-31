-- Add up migration script here
insert into cpo(network, name, slug_name, extra)
Values('fa028728-6950-4892-9111-03ae70e59381', 'pfalzwerke', 'Pfalzwerke', 
'{"powerAC": 22,"powerDC": 50,"expectAC": 3,"expectDC": 3}');


-- ionity
update cpo
set extra = '{"powerAC": 22,"powerDC": 150,"expectAC": 0,"expectDC": 3}'
where 
    network = '429bf694-699e-4156-8535-1554bb11f64e';

-- enable
-- EWE/SWB
-- elli
update cpo
set is_enabled = true
where 
    network = '606a4593-576a-4dd3-a43b-e890063314d0' or
    network = 'da798111-a97b-4262-b091-966330ae0d49' or
    network = '429bf694-699e-4156-8535-1554bb11f64e';

-- aral
update cpo
set slug_name = 'aralpulse'
where 
    network = '2ac29d9c-2ce9-4d8e-a8b8-297aedf4ea2e';

-- eweswb
update cpo
set slug_name = 'eweswb'
where 
    network = '606a4593-576a-4dd3-a43b-e890063314d0';

-- fix slug name for gpjoule
update cpo
set slug_name = 'GP Joule'
where 
    network = '3ad0f93f-8f41-476d-b214-3b892d6399e0';