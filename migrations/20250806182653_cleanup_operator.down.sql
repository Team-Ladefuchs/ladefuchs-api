-- Add down migration script here
alter table operator add column supported_types chargetype [] not null default array[
    'AC', 'DC'
]::chargetype [];

alter table operator
add column if not exists ccs_plug_count integer not null default 0,
add column if not exists power_ac integer not null default 0,
add column if not exists power_dc integer not null default 0,
add column if not exists type2_plug_count integer not null default 0;
