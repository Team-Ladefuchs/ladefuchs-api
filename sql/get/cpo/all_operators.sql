select 
    id, network, 
    pub_network,
    slug_name, name,
    is_enabled,
    power_ac, power_dc,
    updated,
    hide as "hide!",
    supported_types as "supported_types: Vec<ChargeType>",
    url,
	image
from operator
order by operator.name
