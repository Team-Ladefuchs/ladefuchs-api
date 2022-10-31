select
    concat('cpo-', lower(name)) as "identifier!",
    lower(name) as "name!",
    slug_name as display_name
from cpo
where $2 or is_enabled = $1
order by cpo.name
