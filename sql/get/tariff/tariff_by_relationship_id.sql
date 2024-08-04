select
    id,
    slug_name,
    monthly_fee,
    url,
    relationship_id,
    provider_name,
    provider_customer_only,
    standard,
    image
from tariff where relationship_id = $1
