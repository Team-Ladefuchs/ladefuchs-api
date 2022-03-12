-- Add up migration script here
insert into vehicle(
    uuid,
    pub_uuid,
    name,
    vehicle_type
)
values (
    'c2906db7-6efd-474f-bba5-7e128aa0477f',
    '406db33a-4420-46c0-a792-a7d9d974fa63',
    '',
    'Empty'
);

insert into vehicle(
    uuid, 
    pub_uuid,
    name,
    vehicle_type
)
values (
    'c1fd1277-5d77-416b-bb25-84bd21f57963', 
    '1f2761fb-7310-450e-ad45-b45636925a9b', 
    'ioniq', 
    'Car'
);

