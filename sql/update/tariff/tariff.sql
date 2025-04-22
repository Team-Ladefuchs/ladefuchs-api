UPDATE tariff
SET
    slug_name = $2,
    monthly_fee = $3,
    url = $4,
    updated = now(),
    provider_name = $5,
    provider_customer_only = $6,
    standard = $7,
    ad_hoc = $8,
    provider_id = $9,
    brand_only = $10,
    internal_name = $11
WHERE id = $1
