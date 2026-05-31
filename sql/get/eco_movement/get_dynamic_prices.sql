WITH price_components_parsed AS (
    SELECT
        p.id AS price_id,
        (pc ->> 'price_excl_vat')::double precision AS price_excl_vat,
        (entry -> 'restrictions' ->> 'min_duration')::int AS min_duration,
        ((pc ->> 'vat')::double precision) + 100 AS vat,
        pc ->> 'price_type' AS price_type,
        entry -> 'restrictions' ->> 'start_time' AS start_time,
        entry -> 'restrictions' ->> 'end_time' AS end_time,
        (entry -> 'restrictions' -> 'day_of_week')::text AS day_of_week_text,
        entry -> 'restrictions' ->> 'start_date' AS start_date,
        entry -> 'restrictions' ->> 'end_date' AS end_date
    FROM eco_movement.price AS p,
        json_array_elements(p.elements) AS entry,
        json_array_elements(entry -> 'price_components') AS pc
),

energy_prices AS (
    SELECT
        price_id,
        (price_excl_vat * (vat / 100)) AS kw_price_with_vat,
        start_time,
        end_time,
        day_of_week_text,
        start_date,
        end_date
    FROM price_components_parsed
    WHERE price_type = 'ENERGY'
),

parking_prices AS (
    SELECT
        price_id,
        min_duration,
        CASE
            WHEN price_excl_vat > 0
                THEN (price_excl_vat * (vat / 100))
            ELSE 0
        END AS price_parking_time,
        start_time,
        end_time
    FROM price_components_parsed
    WHERE price_type = 'PARKING_TIME' AND min_duration > 0
),

aggregated AS (
    SELECT DISTINCT
        cp.location_id AS eco_location_id,
        p.tariff_id,
        tt.product_id,
        e.kw_price_with_vat,
        pt.price_parking_time,
        pt.min_duration,
        e.start_time,
        e.end_time,
        e.day_of_week_text,
        e.start_date,
        e.end_date,
        CASE
            WHEN c.power_type IN ('ac1_phase', 'ac3_phase')
                THEN 'AC'::chargetype
            ELSE 'DC'::chargetype
        END AS power_type
    FROM eco_movement.price AS p
    INNER JOIN energy_prices AS e ON p.id = e.price_id
    LEFT JOIN parking_prices AS pt ON p.id = pt.price_id
        AND COALESCE(e.start_time, '') = COALESCE(pt.start_time, '')
        AND COALESCE(e.end_time, '') = COALESCE(pt.end_time, '')
    INNER JOIN eco_movement.connector_price AS cp ON p.id = cp.pricing_id
    INNER JOIN eco_movement.connector AS c
        ON cp.connector_id = c.id AND cp.evse_uid = c.evse_uid
    INNER JOIN eco_movement.tariff AS tt ON p.tariff_id = tt.id
    WHERE e.kw_price_with_vat > 0
)

SELECT
    a.eco_location_id AS "eco_location_id!",
    op.id AS "operator_id!",
    tf.id AS "tariff_id!",
    a.kw_price_with_vat AS "price!",
    a.power_type AS "power_type!: ChargeType",
    a.min_duration AS blocking_fee_start,
    a.price_parking_time AS blocking_fee,
    a.start_time,
    a.end_time,
    a.start_date,
    a.end_date,
    a.product_id,
    CASE
        WHEN a.day_of_week_text IS NOT NULL AND a.day_of_week_text != 'null'
        THEN a.day_of_week_text::json
        ELSE NULL
    END AS day_of_week_json
FROM aggregated AS a
INNER JOIN eco_movement.location AS l ON a.eco_location_id = l.id
INNER JOIN eco_movement.operator AS eo ON l.operator_id = eo.id
INNER JOIN public.operator AS op ON op.network = eo.id
INNER JOIN public.tariff AS tf ON tf.relationship_id = a.tariff_id
WHERE NOT EXISTS (
    SELECT 1
    FROM charge_price_blocklist cpb
    WHERE cpb.operator_id = op.id AND cpb.tariff_id = tf.id
);
