select
    tariff.pub_tariff_id as identifier,
    tariff.slug_name as tariff_name,
    msp.name as provider,
    msp.pub_msp_id as msp,
    case
        when tariff.alternative_operator_name is not null then tariff.alternative_operator_name
        else msp.legacy_id
        end as "legacy_id!",
    charge_price.price,
    tariff.monthly_fee as monthly_fee,
    tariff.note,
    charge_price.updated,
    charge_price.blocking_fee_start,
    case 
        when tariff_image.soft_delete = false then $3 || 'img/card/' || tariff_image.checksum
        else null
    end as image,
    case 
        when tariff_image.is_ac_hoc = false then tariff.url 
        else null
    end as tariff_url
from charge_price join cpo on cpo.id = charge_price.cpo_id
                  join tariff on tariff.id = charge_price.tariff_id
                  left join tariff_image on image = tariff_image.id
                  join msp on tariff.msp_id = msp.id
where
        cpo.id = $1 and
        charge_price.c_type = $2 and
        cpo.is_enabled and
        cpo.hide = false
order by price, tariff.slug_name;
