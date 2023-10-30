select 
    id, network, 
    pub_network,
    slug_name, name,
    is_enabled,
    power_ac, power_dc,
    updated,
    EXISTS (SELECT cpo_id FROM charge_price WHERE cpo_id = operator.id) as "hide!",
    supported_types as "supported_types: Vec<ChargeType>",
    url,
	image
from operator
where network = $1 or lower(name) = lower($2)
order by operator.name
