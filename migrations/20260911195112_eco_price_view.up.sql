-- Add up migration script here
-- Precompute the modal Eco-Movement gross ENERGY price per
-- (operator, tariff relationship, charge type). Replaces the expensive
-- JSON aggregation that the admin prices page and feedback report run per request.

CREATE MATERIALIZED VIEW public.eco_price AS
WITH usage AS (
    SELECT
        cp.pricing_id,
        CASE
            WHEN c.power_type::text LIKE 'ac%' THEN 'AC'
            WHEN c.power_type::text = 'dc' THEN 'DC'
        END AS charge_type,
        p.tariff_id,
        cl.operator_id,
        count(*) AS weight
    FROM eco_movement.connector_price AS cp
    INNER JOIN eco_movement.connector AS c
        ON c.id = cp.connector_id
       AND c.evse_uid = cp.evse_uid
    INNER JOIN eco_movement.price AS p
        ON p.id = cp.pricing_id
    INNER JOIN public.charging_location AS cl
        ON cl.eco_movement_id = cp.location_id
    WHERE c.power_type::text LIKE 'ac%'
       OR c.power_type::text = 'dc'
    GROUP BY 1, 2, 3, 4
),
expanded AS (
    SELECT
        u.operator_id,
        u.tariff_id,
        u.charge_type,
        (component->>'price_excl_vat')::numeric
            * (1 + COALESCE((component->>'vat')::numeric, 0) / 100) AS price,
        sum(u.weight) AS occurrence_count
    FROM usage AS u
    INNER JOIN eco_movement.price AS p
        ON p.id = u.pricing_id
    CROSS JOIN LATERAL json_array_elements(p.elements) AS element
    CROSS JOIN LATERAL json_array_elements(element->'price_components') AS component
    WHERE u.charge_type IS NOT NULL
      AND component->>'price_type' = 'ENERGY'
    GROUP BY 1, 2, 3, 4
)
SELECT DISTINCT ON (operator_id, tariff_id, charge_type)
    operator_id,
    tariff_id,
    charge_type,
    price
FROM expanded
ORDER BY operator_id, tariff_id, charge_type, occurrence_count DESC, price DESC;

CREATE UNIQUE INDEX eco_price_key
    ON public.eco_price (operator_id, tariff_id, charge_type);
