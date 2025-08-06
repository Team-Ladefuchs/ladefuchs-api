update operator
set
    network = $1,
    name = $2,
    slug_name = $3,
    standard = $4,
    evse_id = $5
where id = $6
