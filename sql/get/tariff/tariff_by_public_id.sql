select
    id,
    slug_name,
    monthly_fee,
    url,
    relationship_id,
    provider_name,
    provider_customer_only,
    standard,
    image,
    ad_hoc,
    provider_id,
    brand_only
from tariff where pub_tariff_id = $1
