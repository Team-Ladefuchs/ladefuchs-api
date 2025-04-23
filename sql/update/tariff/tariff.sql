UPDATE tariff
SET
    slug_name = $2,
    monthly_fee = $3,
    provider_name = $4,
    provider_customer_only = $5,
    standard = $6,
    ad_hoc = $7,
    relationship_id = $8
WHERE id = $1
