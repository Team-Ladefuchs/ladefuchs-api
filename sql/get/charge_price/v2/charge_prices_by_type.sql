select
    charge_price.c_type as "c_type: ChargeType",
    tariff.pub_tariff_id as identifier,
    tariff.slug_name as tariff_name,
    tariff.provider_name as provider,
    tariff.provider_id as msp,
    charge_price.price,
    tariff.monthly_fee,
    tariff.note,
    tariff.provider_name as "legacy_id!",
    charge_price.updated,
    charge_price.blocking_fee_start,
    charge_price.blocking_fee,
    case
        when image.soft_delete = false then $3 || 'img/card/' || image.checksum
    end as image,
    case
        when image.is_ad_hoc = true then null
        else tariff.url
    end as tariff_url
from charge_price inner join operator on charge_price.operator_id = operator.id
inner join tariff on charge_price.tariff_id = tariff.id
left join image on tariff.image = image.id
where
    operator.id = $1
    and charge_price.c_type = $2
    and operator.standard
    and tariff.standard
    and tariff.hide = false

order by price, tariff.slug_name;
