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
ON CONFLICT DO NOTHING
RETURNING id;
