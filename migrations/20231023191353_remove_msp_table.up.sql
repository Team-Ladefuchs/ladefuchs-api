-- Add up migration script here
alter table tariff add column if not exists provider_name text not null check ( provider_name <> '' ) default 'provider_name',
add column if not exists provider_id uuid not null default gen_random_uuid(),
add column if not exists override_standard bool not null default false,
add column if not exists provider_customer_only bool not null default false;

alter table if exists tariff rename column is_enabled to standard;

update tariff 
set standard = false;

CREATE OR REPLACE FUNCTION replace_msp_id()
    RETURNS VOID AS $$
DECLARE
    record RECORD;
    msp_name text;
    msp_pub_id uuid;
BEGIN
    -- Check if msp table exists
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'msp') THEN
        -- Iterate over every row in the tariff table
        FOR record IN (SELECT id, msp_id FROM tariff) LOOP
            SELECT name, pub_msp_id INTO msp_name, msp_pub_id FROM msp WHERE id = record.msp_id;
            -- Update tariff table with msp information
            UPDATE tariff
            SET provider_name = msp_name,
                provider_id = msp_pub_id
            WHERE id = record.id;
        END LOOP;
    ELSE
        -- Handle the case where msp table does not exist
        RAISE NOTICE 'msp table does not exist';
    END IF;
END;
$$ LANGUAGE PLpgSQL;

select replace_msp_id();
drop function if exists replace_msp_id;

alter table tariff drop column if exists msp_id;

drop table if exists msp cascade;
drop table if exists msp_cpo cascade;
