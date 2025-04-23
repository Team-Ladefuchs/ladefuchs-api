select
    operator.id,
    operator.network,
    operator.pub_network,
    operator.slug_name,
    operator.name,
    operator.standard,
    operator.power_ac,
    operator.power_dc,
    operator.updated,
    operator.supported_types as "supported_types: Vec<ChargeType>",
    operator.url,
    operator.image,
    operator.ccs_plug_count,
    operator.type2_plug_count,
    operator.evse_id,
    not exists (
        select operator_id from charge_price
        where operator_id = operator.id
    ) as "hide!",
    operator.ccs_plug_count + operator.type2_plug_count as "sum_plug_count!"
from operator
where
    operator.network = $1
    or lower(operator.name) = lower($2)
order by operator.name
