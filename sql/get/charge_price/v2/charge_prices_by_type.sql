select
    charge_price.c_type as "c_type: ChargeType",
    tariff.pub_tariff_id as identifier,
    tariff.slug_name as tariff_name,
    tariff.provider_name as provider,
    tariff.provider_id as msp,
    charge_price.price,
    tariff.monthly_fee as monthly_fee,
    tariff.note,
    case
        when
            tariff.alternative_operator_name is not null
            then tariff.alternative_operator_name
        else tariff.provider_name
    end as "legacy_id!",
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
    charge_price.blocking_fee
from charge_price join operator on operator.id = charge_price.operator_id
join tariff on tariff.id = charge_price.tariff_id
left join image on tariff.image = image.id
where
    operator.id = $1
    and charge_price.c_type = $2
    and operator.standard
    and tariff.standard or tariff.override_standard
    and tariff.hide = false

order by price, tariff.slug_name;
