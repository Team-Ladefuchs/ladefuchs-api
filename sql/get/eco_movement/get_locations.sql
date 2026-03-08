SELECT
    l.id AS "eco_movement_id!",
    op.id AS "operator_id!",
    (l.value -> 'coordinates' ->> 'latitude')::double precision AS "latitude!",
    (l.value -> 'coordinates' ->> 'longitude')::double precision AS "longitude!",
    l.value ->> 'address' AS address,
    l.value ->> 'city' AS city,
    l.value ->> 'postal_code' AS postal_code
FROM eco_movement.location AS l
INNER JOIN eco_movement.operator AS eo ON l.operator_id = eo.id
INNER JOIN public.operator AS op ON op.network = eo.id
WHERE (l.value -> 'coordinates' ->> 'latitude') IS NOT NULL
  AND (l.value -> 'coordinates' ->> 'longitude') IS NOT NULL;
