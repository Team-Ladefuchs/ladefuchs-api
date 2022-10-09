insert into msp(name, legacy_id) values ($1, $2) 
on conflict(name) 
    DO update 
        set 
            name = excluded.name,
            legacy_id = excluded.legacy_id
RETURNING id
