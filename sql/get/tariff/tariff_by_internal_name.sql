SELECT tariff.id as tariff_id
FROM tariff JOIN msp m on tariff.msp_id = m.id
WHERE internal_name ilike $1;