select 
    ( select updated from  charge_price where is_protected = false Limit 1 ) as last_import, 
    count(cpo_id) as prices
from charge_price
rustu
