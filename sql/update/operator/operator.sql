update operator
set
    name = $2,
    slug_name = $3,
    standard = $4,
    supported_types = $5::CHARGETYPE [],
    power_ac = $6,
    power_dc = $7,
    updated = now()
where network = $1
