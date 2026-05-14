WITH
update_by_product AS (
    UPDATE eco_movement.tariff SET
        name = $1,
        description = $2,
        subscription_type = $3,
        type = $4,
        subscription_fee_excl_vat = $5,
        currency = $6,
        provider_name = $7
    WHERE product_id = $9
    RETURNING id
),
update_by_name AS (
    UPDATE eco_movement.tariff SET
        product_id = $9,
        description = $2,
        subscription_type = $3,
        type = $4,
        subscription_fee_excl_vat = $5,
        currency = $6
    WHERE product_id IS NULL
      AND name = $1
      AND provider_name = $7
      AND NOT EXISTS (SELECT 1 FROM update_by_product)
    RETURNING id
),
ins AS (
    INSERT INTO eco_movement.tariff (
        name,
        description,
        subscription_type,
        type,
        subscription_fee_excl_vat,
        currency,
        provider_name,
        id,
        product_id
    )
    SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9
    WHERE NOT EXISTS (SELECT 1 FROM update_by_product)
      AND NOT EXISTS (SELECT 1 FROM update_by_name)
    RETURNING id
)
SELECT id AS "id!" FROM update_by_product
UNION ALL
SELECT id AS "id!" FROM update_by_name
UNION ALL
SELECT id AS "id!" FROM ins
LIMIT 1;
