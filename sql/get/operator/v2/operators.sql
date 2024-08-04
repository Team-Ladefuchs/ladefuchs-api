select
    pub_network as identifier,
    slug_name as display_name,
    GREATEST(image.updated, operator.updated) as "updated!",
    supported_types as "types: Vec<ChargeType>",
    case
        when image.soft_delete = false then $2 || 'img/cpo/' || image.checksum
        else null
    end as image
from operator left join image on operator.image = image.id
where
    standard = $1
    and exists (
        select operator_id from charge_price where operator_id = operator.id
    )
order by operator.name
