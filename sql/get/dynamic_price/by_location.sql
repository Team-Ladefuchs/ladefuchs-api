WITH nearby_locations AS (
    SELECT
        cl.id AS location_id,
        cl.eco_movement_id,
        cl.operator_id,
        ST_Y(cl.geo::geometry) AS latitude,
        ST_X(cl.geo::geometry) AS longitude,
        cl.address,
        cl.city,
        ST_Distance(cl.geo, ST_MakePoint($1, $2)::geography) AS distance,
        o_cpo.pub_network AS cpo_id,
        o_cpo.slug_name AS cpo_name
    FROM charging_location AS cl
    INNER JOIN operator AS o_cpo ON cl.operator_id = o_cpo.id
    WHERE ST_DWithin(cl.geo, ST_MakePoint($1, $2)::geography, $3)
),

prices_with_rank AS (
    SELECT
        nl.location_id,
        nl.latitude,
        nl.longitude,
        nl.address,
        nl.city,
        nl.distance,
        nl.cpo_id,
        nl.cpo_name,
        dp.tariff_id,
        dp.c_type,
        dp.price,
        dp.blocking_fee_start,
        dp.blocking_fee,
        dp.valid_from,
        dp.valid_until,
        t.pub_tariff_id AS tariff_public_id,
        t.slug_name AS tariff_name,
        t.provider_id,
        t.provider_name,
        ROW_NUMBER() OVER (
            PARTITION BY nl.location_id, dp.operator_id, dp.tariff_id, dp.c_type
            ORDER BY
                CASE WHEN (dp.start_time IS NOT NULL OR array_length(dp.day_of_week, 1) != 7) AND dp.valid_from IS NOT NULL THEN 0 ELSE 1 END,
                CASE WHEN dp.valid_from IS NOT NULL THEN 0 ELSE 1 END,
                CASE WHEN (dp.start_time IS NOT NULL OR array_length(dp.day_of_week, 1) != 7) THEN 0 ELSE 1 END
        ) AS rn
    FROM nearby_locations AS nl
    INNER JOIN location_dynamic_price AS ldp ON nl.location_id = ldp.location_id
    INNER JOIN dynamic_charge_price AS dp ON ldp.dynamic_price_id = dp.id
    INNER JOIN tariff AS t ON dp.tariff_id = t.id
    WHERE
        $5::day_of_week = ANY(dp.day_of_week)
        AND (
            dp.start_time IS NULL
            OR $4::time BETWEEN dp.start_time AND dp.end_time
        )
        AND (dp.valid_from IS NULL OR $6::date >= dp.valid_from)
        AND (dp.valid_until IS NULL OR $6::date <= dp.valid_until)
)

SELECT
    location_id AS "location_id!",
    latitude AS "latitude!",
    longitude AS "longitude!",
    address,
    city,
    distance AS "distance!",
    cpo_id AS "cpo_id!",
    cpo_name AS "cpo_name!",
    tariff_public_id AS "tariff_id!",
    tariff_name AS "tariff_name!",
    c_type AS "charging_mode!: ChargeType",
    price AS "price_per_kwh!",
    blocking_fee_start AS "blocking_fee_start!",
    blocking_fee AS "blocking_fee!",
    valid_from,
    valid_until,
    provider_id AS "provider_id!",
    provider_name AS "provider_name!"
FROM prices_with_rank
WHERE rn = 1
ORDER BY distance, tariff_name;
