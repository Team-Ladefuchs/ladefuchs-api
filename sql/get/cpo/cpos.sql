select 
    id, network, 
    pub_network,
    slug_name, name,
    is_enabled,
    power_ac, power_dc,
    updated,
    hide as "hide!",
    supported_types as "supported_types: Vec<ChargeType>",
    (select url from cpo_cache where cpo_cache.network = cpo.network) as url
from cpo
order by cpo.name
