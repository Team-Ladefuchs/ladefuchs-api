select
    tariff.pub_tariff_id as identifier,
    tariff.slug_name as tariff_name,
    msp.name as provider,
    case
        when tariff.alternative_operator_name is not null then tariff.alternative_operator_name
        else msp.legacy_id
        end as "legacy_id!",
    charge_price.price,
    tariff.monthly_fee as monthly_fee,
    tariff.note,
    charge_price.updated,
    charge_price.blocking_fee_start,
    $3 || 'img/card/' || tariff_image.checksum as image,
    tariff.url as tariff_url
from charge_price join cpo on cpo.id = charge_price.cpo_id
                  join tariff on tariff.id = charge_price.tarif_id
                  left join tariff_image on image = tariff_image.id
                  join msp on tariff.msp_id = msp.id
where
        (cpo.name ilike $1 or cpo.pub_network::text = $1 ) and
        cpo.is_enabled and
        charge_price.c_type::text ilike $2
order by price, msp.name desc, tariff.slug_name;
