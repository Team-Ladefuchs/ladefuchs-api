select
    id,
    network,
    pub_network,
    slug_name,
    name,
    standard,
    power_ac,
    power_dc,
    updated,
    not exists (
        select operator_id from charge_price where operator_id = operator.id
    ) as "hide!",
    supported_types as "supported_types: Vec<ChargeType>",
    url,
    image,
    ccs_plug_count,
    type2_plug_count,
    ccs_plug_count + type2_plug_count as "sum_plug_count!"
from operator
where network = $1 or lower(name) = lower($2)
order by operator.name
