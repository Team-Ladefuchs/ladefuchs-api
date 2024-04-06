select 
    id, network, 
    pub_network,
    slug_name, name,
    standard,
    power_ac, power_dc,
    updated,
    NOT EXISTS (SELECT operator_id FROM charge_price WHERE operator_id = operator.id) as "hide!",
    supported_types as "supported_types: Vec<ChargeType>",
    url,
	image,
	ccs_plug_count,
    type2_plug_count,
	ccs_plug_count + type2_plug_count as "sum_plug_count!"
from operator
WHERE ccs_plug_count > 0 or type2_plug_count > 0
order by operator.name

