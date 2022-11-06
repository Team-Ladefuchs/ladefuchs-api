update cpo 
    set 
        name = $2,
        slug_name = $3,
        is_enabled = $4,
        supported_types = $5,
        power_ac = $6,
        power_dc = $7,
        updated = now()
where cpo.id = $1
