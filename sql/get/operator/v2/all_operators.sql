select
    pub_network as identifier,
    slug_name as display_name,
    GREATEST(image.updated, operator.updated) as "updated!",
    supported_types as "types: Vec<ChargeType>",
    case
        when image.soft_delete = false then $1 || 'img/cpo/' || image.checksum
        else null
    end as image
from operator left join image on operator.image = image.id
where
    exists (
        select operator_id
        from charge_price
        where
            operator_id = operator.id and ccs_plug_count > 0
            or type2_plug_count > 0
    )
order by operator.name
