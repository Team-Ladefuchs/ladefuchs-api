WITH operator_data AS (
    SELECT id
    FROM
        operator
    WHERE network = $1
),

tariff_data AS (
    SELECT id
    FROM tariff
    WHERE relationship_id = $2
)

INSERT INTO charge_price (
    operator_id, tariff_id, c_type, price, blocking_fee_start, blocking_fee
)
SELECT
    o.id,
    t.id,
    $3,
    $4,
    $5,
    $6
FROM operator_data AS o,
    tariff_data AS t
WHERE
    o.id IS NOT NULL
    AND t.id IS NOT NULL ON CONFLICT (
    operator_id,
    tariff_id,
    c_type
) DO
UPDATE
SET price = excluded.price,
blocking_fee_start = excluded.blocking_fee_start,
blocking_fee = excluded.blocking_fee,
updated = NOW();
