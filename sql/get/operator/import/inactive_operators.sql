select slug_name
from operator
where
    not exists (
        select 1 from charge_price
        where operator_id = id and charge_price.is_protected = false
    )
    and standard = true
