insert into msp(msp_id, name, legacy_id) values ($1, $2, $3) 
on conflict(name) 
    DO update 
        set 
            name = excluded.name,
            msp_id = excluded.msp_id, 
            legacy_id = excluded.legacy_id
RETURNING id