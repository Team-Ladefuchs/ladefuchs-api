WITH price_components_parsed AS (
    SELECT
        p.id AS price_id,
        (pc ->> 'price_excl_vat')::double precision AS price_excl_vat,
        (entry -> 'restrictions' ->> 'start_date')::timestamp AS start_date,
        (entry -> 'restrictions' ->> 'min_duration')::int AS min_duration,
        ((pc ->> 'vat')::double precision) + 100 AS vat,
        pc ->> 'price_type' AS price_type
    FROM eco_movement.price AS p,
        json_array_elements(p.elements) AS entry,
        json_array_elements(entry -> 'price_components') AS pc
    WHERE entry -> 'restrictions' -> 'start_time' IS null
),

energy_prices_ranked AS (
    SELECT
        p.id AS price_id,
        pc.start_date,
        (pc.price_excl_vat * (pc.vat / 100)) AS kw_price_with_vat
    FROM eco_movement.price AS p
    INNER JOIN price_components_parsed AS pc ON p.id = pc.price_id
    WHERE pc.price_type = 'ENERGY'
),

parking_prices_ranked AS (
    SELECT
        p.id AS price_id,
        pc.min_duration,
        CASE
            WHEN pc.price_excl_vat > 0
                THEN (pc.price_excl_vat * (pc.vat / 100))
            ELSE 0
        END AS price_parking_time
    FROM eco_movement.price AS p
    INNER JOIN price_components_parsed AS pc ON p.id = pc.price_id
    WHERE pc.price_type = 'PARKING_TIME' AND pc.min_duration > 0
),

aggregated_prices AS (
    SELECT
        p.tariff_id,
        tt.product_id,
        l.operator_id,
        e.kw_price_with_vat,
        pt.price_parking_time,
        pt.min_duration,
        e.start_date,
        c.max_power,
        CASE
            WHEN
                c.power_type IN ('ac1_phase', 'ac3_phase')
                THEN 'AC'::chargetype
            ELSE 'DC'::chargetype
        END AS power_type
    FROM eco_movement.price AS p
    INNER JOIN energy_prices_ranked AS e ON p.id = e.price_id
    LEFT JOIN parking_prices_ranked AS pt ON p.id = pt.price_id
    INNER JOIN eco_movement.connector_price AS cp ON p.id = cp.pricing_id
    INNER JOIN eco_movement.location AS l ON cp.location_id = l.id
    INNER JOIN eco_movement.operator AS o ON l.operator_id = o.id
    INNER JOIN
        eco_movement.connector AS c
        ON cp.connector_id = c.id AND cp.evse_uid = c.evse_uid
    INNER JOIN eco_movement.tariff AS tt ON p.tariff_id = tt.id
    GROUP BY
        tariff_id,
        tt.product_id,
        operator_id,
        power_type,
        kw_price_with_vat,
        price_parking_time,
        min_duration,
        e.start_date,
        c.max_power
),

prices_with_rn AS (
    SELECT
        tariff_id,
        product_id,
        operator_id,
        power_type,
        kw_price_with_vat,
        price_parking_time,
        min_duration,
        max_power,
        start_date,
        row_number()
            OVER (
                PARTITION BY operator_id, tariff_id, max_power, power_type
                ORDER BY start_date DESC
            )
        AS rn
    FROM aggregated_prices
),

ranked_price AS (
    SELECT
        *,
        row_number() OVER (
            PARTITION BY operator_id, tariff_id, power_type
            ORDER BY max_power DESC
        ) AS outer_rn
    FROM (
        SELECT * FROM prices_with_rn
        WHERE rn = 1
    ) AS inner_rn
)

SELECT
    tf.id AS tariff_id,
    op.id AS operator_id,
    min_duration AS blocking_fee_start,
    kw_price_with_vat AS "price_kw!",
    power_type AS "power_type!: ChargeType",
    price_parking_time AS blocking_fee,
    product_id
FROM ranked_price
INNER JOIN public.operator AS op ON op.network = operator_id
INNER JOIN public.tariff AS tf ON tf.relationship_id = tariff_id
WHERE outer_rn = 1
  AND kw_price_with_vat > 0
  AND not exists (select operator_id
                  from charge_price_blocklist cpb
                  where cpb.operator_id = op.id and cpb.tariff_id = tf.id);
