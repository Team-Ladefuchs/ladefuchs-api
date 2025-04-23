WITH price_components_parsed AS (
    SELECT
        p.id AS price_id,
        (pc ->> 'price_excl_vat')::numeric AS price_excl_vat,
        (entry -> 'restrictions' ->> 'start_date')::timestamp AS start_date,
        pc ->> 'price_type' AS price_type,
        (entry -> 'restrictions' ->> 'min_duration') AS min_duration
    FROM eco_movement.price AS p,
        json_array_elements(p.elements) AS entry,
        json_array_elements(entry -> 'price_components') AS pc
    WHERE entry -> 'restrictions' -> 'start_time' IS null
),

energy_prices_ranked AS (
    SELECT
        p.id AS price_id,
        pc.price_excl_vat AS price_energy,
        pc.start_date
    FROM eco_movement.price AS p
    INNER JOIN price_components_parsed AS pc ON p.id = pc.price_id
    WHERE pc.price_type = 'ENERGY'
),

parking_prices_ranked AS (
    SELECT
        p.id AS price_id,
        pc.price_excl_vat AS price_parking_time,
        pc.min_duration
    FROM eco_movement.price AS p
    INNER JOIN price_components_parsed AS pc ON p.id = pc.price_id
    WHERE pc.price_type = 'PARKING_TIME'
),

aggregated_prices AS (
    SELECT
        p.tariff_id,
        l.operator_id,
        e.price_energy AS price_kw,
        pt.price_parking_time,
        pt.min_duration,
        e.start_date,
        c.max_power,
        CASE
            WHEN c.power_type IN ('ac1_phase', 'ac3_phase') THEN 'ac'
            ELSE c.power_type::text
        END AS power_type
    --         ROW_NUMBER() OVER (PARTITION BY c.max_power ORDER BY e.start_date DESC) AS rn
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
        operator_id,
        power_type,
        price_kw,
        price_parking_time,
        min_duration,
        e.start_date,
        c.max_power
),

prices_with_rn AS (
    SELECT
        tariff_id,
        operator_id,
        power_type,
        price_kw,
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
    tariff_id,
    operator_id,
    power_type,
    price_kw,
    price_parking_time,
    min_duration,
    max_power
FROM ranked_price
WHERE outer_rn = 1
