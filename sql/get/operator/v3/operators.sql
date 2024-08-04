select
    pub_network as identifier,
    slug_name as name,
    GREATEST(image.updated, operator.updated) as "updated!",
    supported_types as "charging_modes: Vec<ChargeType>",
    standard as is_standard,
    case
        when image.soft_delete = false then $2 || 'image/' || image.checksum
        else null
    end as image_url,
    url as website_url
from operator left join image on operator.image = image.id
where
    standard = $1
    and exists (
        select operator_id from charge_price where operator_id = operator.id
    )
order by operator.name
