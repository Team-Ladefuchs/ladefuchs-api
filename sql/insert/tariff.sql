INSERT INTO tariff
(
    relationship_id,
    slug_name,
    monthly_fee,
    url,
    internal_name,
    image,
    provider_name,
    provider_customer_only,
    standard,
    ad_hoc,
    provider_id
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
RETURNING id
