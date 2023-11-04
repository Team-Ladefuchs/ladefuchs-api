select 
    ( select updated from  charge_price where is_protected = false Limit 1 ) as last_import, 
    count(operator_id) as prices
from charge_price
