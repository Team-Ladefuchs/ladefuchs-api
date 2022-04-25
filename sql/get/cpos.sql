select 
    id, network, 
    pub_network, name, 
    slug_name, is_enabled,
    power_ac, power_dc,
    expect_ac, expect_dc
from cpo
order by cpo.name