select
    operator.id,
    operator.network,
    operator.pub_network,
    operator.slug_name,
    operator.name,
    operator.standard,
    operator.updated,
    operator.url,
    operator.image,
    operator.evse_id,
    not exists (
        select operator_id from charge_price
        where operator_id = operator.id
    ) as "hide!"
from operator
where
    operator.network = $1
    or lower(operator.name) = lower($2)
order by operator.name
