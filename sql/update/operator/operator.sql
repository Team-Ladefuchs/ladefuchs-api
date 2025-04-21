update operator
set
	network = $1,
    name = $2,
    slug_name = $3,
    standard = $4,
    supported_types = $5::CHARGETYPE [],
    power_ac = $6,
    power_dc = $7,
    updated = now(),
    evse_id = $8
where id = $9
