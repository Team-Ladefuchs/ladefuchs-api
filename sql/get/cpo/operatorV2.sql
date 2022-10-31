select
    pub_network as identifier,
    slug_name as display_name,
    updated,
    case
        when expect_ac > 0 and expect_dc > 0 then array['ac', 'dc'] 
        when expect_ac > 0 then array['ac']
        when expect_dc > 0 then array['dc']
        else array ['']
    end as "types!"
from cpo
where $2 or is_enabled = $1
order by cpo.name
