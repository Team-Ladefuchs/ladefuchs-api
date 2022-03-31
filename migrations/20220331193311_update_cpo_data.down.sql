-- Add down migration script here
update cpo
set slug_name = 'aral'
where 
    network = '2ac29d9c-2ce9-4d8e-a8b8-297aedf4ea2e';


-- eweswb
update cpo
set slug_name = 'ewe'
where 
    network = '606a4593-576a-4dd3-a43b-e890063314d0';

update cpo
set is_enabled = false
where 
    network = '606a4593-576a-4dd3-a43b-e890063314d0' or
    network = 'da798111-a97b-4262-b091-966330ae0d49' or
    network = '429bf694-699e-4156-8535-1554bb11f64e';

delete from cpo
where network = 'fa028728-6950-4892-9111-03ae70e59381';