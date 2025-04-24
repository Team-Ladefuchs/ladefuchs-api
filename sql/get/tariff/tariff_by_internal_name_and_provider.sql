SELECT
    tariff.id,
    tariff.slug_name,
    tariff.monthly_fee,
    tariff.url,
    tariff.relationship_id,
    tariff.provider_name,
    tariff.provider_customer_only,
    tariff.standard,
    tariff.image,
    tariff.ad_hoc,
    tariff.provider_id,
    tariff.brand_only
FROM tariff
WHERE
    tariff.relationship_id = $3
    OR (tariff.internal_name = $1 AND lower(tariff.provider_name) = lower($2))
LIMIT 1;
