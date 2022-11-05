select
    pub_network as identifier,
    slug_name as display_name,
    updated,
    supported_types as "types: Vec<ChargeType>"
from cpo
where is_enabled = $1 or $2
order by cpo.name
