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
where search @@ websearch_to_tsquery($1) or strict_word_similarity($1, slug_name) > 0.25
