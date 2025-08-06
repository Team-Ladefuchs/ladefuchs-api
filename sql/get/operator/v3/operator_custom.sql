select
    pub_network as identifier,
    slug_name as name,
    standard as is_standard,
    url as website_url,
    array[]::integer [] as "types!",
    GREATEST(image.updated, operator.updated) as "updated!",
    case
        when image.soft_delete = false then $1 || 'image/' || image.checksum
    end as image_url
from operator left join image on operator.image = image.id
where
    (
        standard
        and exists (
            select operator_id from charge_price
            where operator_id = operator.id
        )
        or pub_network = ANY($2)
    ) and pub_network != ALL($3)
order by operator.name
