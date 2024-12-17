update tariff
set
    note = $2,
    internal_name = $3,
    hide = $4,
    url = $5
where id = $1
