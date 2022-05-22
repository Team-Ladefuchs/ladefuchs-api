SELECT tarif.id as tarif_id
FROM tarif JOIN msp m on tarif.msp_id = m.id
WHERE internal_name ilike $1;