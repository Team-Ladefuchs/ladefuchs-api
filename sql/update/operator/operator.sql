update operator
set
    network = $1,
    name = $2,
    slug_name = $3,
    standard = $4,
    supported_types = $5::CHARGETYPE [],
    power_ac = $6,
    power_dc = $7,
    evse_id = $8
where id = $9
ON CONFLICT (name)
DO UPDATE SET
    network = EXCLUDED.network,
    slug_name = EXCLUDED.slug_name,
    standard = EXCLUDED.standard,
    supported_types = EXCLUDED.supported_types,
    power_ac = EXCLUDED.power_ac,
    power_dc = EXCLUDED.power_dc,
    evse_id = EXCLUDED.evse_id;
