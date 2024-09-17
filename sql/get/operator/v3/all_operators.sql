select
    pub_network as identifier,
    slug_name as name,
    supported_types as "charging_modes: Vec<ChargeType>",
    standard as is_standard,
    url as website_url,
    GREATEST(image.updated, operator.updated) as "updated!",
    case
        when image.soft_delete = false then $1 || 'image/' || image.checksum
    end as image_url
from operator left join image on operator.image = image.id
where
    exists (
        select operator_id
        from charge_price
        where
            operator_id = operator.id and (ccs_plug_count > 0
            or type2_plug_count > 0)
    ) and operator.slug_name not ilike 'tesla%'
order by operator.name
