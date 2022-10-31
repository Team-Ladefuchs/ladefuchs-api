select id, slug_name, monthly_fee, url, relationship_id, msp_id
from tariff where relationship_id = $1
