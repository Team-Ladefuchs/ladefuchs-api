UPDATE msp
    SET 
        name = $2,
        legacy_id = $3
where id = $1