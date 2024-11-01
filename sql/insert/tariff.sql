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
    ad_hoc
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
ON CONFLICT (relationship_id)
DO UPDATE
SET
slug_name = excluded.slug_name,
monthly_fee = excluded.monthly_fee,
provider_name = excluded.provider_name,
provider_customer_only = excluded.provider_customer_only,
standard = excluded.standard,
ad_hoc = excluded.ad_hoc,
url = excluded.url
RETURNING id
