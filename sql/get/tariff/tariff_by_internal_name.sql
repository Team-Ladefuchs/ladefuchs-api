SELECT tariff.id AS tariff_id
FROM tariff
WHERE lower(internal_name) = lower($1);
