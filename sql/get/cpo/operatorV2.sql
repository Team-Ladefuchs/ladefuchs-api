select
    pub_network as identifier,
    slug_name as display_name,
    updated,
    case
        when expect_ac > 0 and expect_dc > 0 then array['AC', 'DC'] 
        when expect_ac > 0 then array['AC']
        when expect_dc > 0 then array['DC']
        else array ['']
    end as "types!"
from cpo
where $2 or is_enabled = $1
order by cpo.name
