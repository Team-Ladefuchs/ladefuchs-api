select updated as last_import, count(cpo_id) as prices
from charge_price
GROUP BY updated
limit 1
