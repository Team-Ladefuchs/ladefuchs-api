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
    provider_id
from tariff where relationship_id = $1
