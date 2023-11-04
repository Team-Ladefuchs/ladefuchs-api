select slug_name
from operator
where not exists (select operator_id from charge_price where operator_id = id) and standard = true
