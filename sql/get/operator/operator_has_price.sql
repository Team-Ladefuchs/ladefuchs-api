select operator_id 
from charge_price 
where operator_id = $1 limit 1
