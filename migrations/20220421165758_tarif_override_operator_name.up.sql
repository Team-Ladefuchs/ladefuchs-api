-- Add down migration script here
alter table tarif add COLUMN alternative_operator_name text;

-- ADAC e-CHARGE tarif
update tarif 
    set alternative_operator_name = 'adac' 
    where relationship_id = '36c2017a-097a-4072-9a58-6ef904b6173d';