select
    pub_network as identifier,
    slug_name as display_name,
	GREATEST(image.updated, operator.updated) as "updated!",
    supported_types as "types: Vec<ChargeType>",
	is_enabled as standard,
	case 
        when image.soft_delete = false then $3 || 'img/cpo/' || image.checksum
        else null
    end as image,
	url
from operator left join image on operator.image = image.id
where $2 or (is_enabled = $1 and hide != $1)
order by operator.name
