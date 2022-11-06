select
    pub_network as identifier,
    slug_name as display_name,
    updated,
    supported_types as "types: Vec<ChargeType>"
from cpo
where $2 or (is_enabled = $1 and hide != $1)
order by cpo.name
