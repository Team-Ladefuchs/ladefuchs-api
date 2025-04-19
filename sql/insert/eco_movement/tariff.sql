WITH ins AS (
    INSERT INTO eco_movement.tariff (
        name,
        description,
        subscription_type,
        type,
        subscription_fee_excl_vat,
        currency,
        provider_name
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    ON CONFLICT (name, provider_name) DO NOTHING
    RETURNING id
)

SELECT id AS "id!" FROM ins
UNION ALL
SELECT id AS "id!" FROM eco_movement.tariff
WHERE name = $1 AND provider_name = $7
LIMIT 1;
