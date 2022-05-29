UPDATE msp
    SET 
        name = $1,
        legacy_id = $2
where id = $1