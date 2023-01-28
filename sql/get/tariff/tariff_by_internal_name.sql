SELECT tariff.id as tariff_id
FROM tariff 
WHERE lower(internal_name) = lower($1);

