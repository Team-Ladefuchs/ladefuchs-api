select
    id, network,
    pub_network, name,
    slug_name, is_enabled,
    power_ac, power_dc,
    expect_ac, expect_dc,
    updated,
    (select url from cpo_cache where cpo_cache.network = cpo.network)
from cpo
order by cpo.name
