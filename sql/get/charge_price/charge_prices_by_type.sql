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
        when image.soft_delete = false then $3 || 'img/card/' || image.checksum
        else null
    end as image,
    case 
        when image.is_ad_hoc = true then null
        else tariff.url 
    end as tariff_url,
	charge_price.blockingfee as blocking_fee
from charge_price join operator on operator.id = charge_price.cpo_id
                  join tariff on tariff.id = charge_price.tariff_id
                  left join image on tariff.image = image.id
                  join msp on tariff.msp_id = msp.id
where
        operator.id = $1 and
        charge_price.c_type = $2 and
        operator.hide = false and
		operator.is_enabled and
		tariff.is_enabled
order by price, tariff.slug_name;
