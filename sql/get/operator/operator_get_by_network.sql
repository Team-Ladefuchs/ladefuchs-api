select 
    id, network, 
    pub_network,
    slug_name, name,
    standard,
    power_ac, power_dc,
    updated,
    EXISTS (SELECT operator_id FROM charge_price WHERE operator_id = operator.id) as "hide!",
    supported_types as "supported_types: Vec<ChargeType>",
    url,
	image,
	ccs_plug_count,
    type2_plug_count
from operator
where network = $1
order by operator.name
