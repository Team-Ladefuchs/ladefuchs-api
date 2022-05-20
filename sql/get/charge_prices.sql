select
    msp.pub_msp_id as identifier,
    tarif.slug_name as name,
    msp.name as provider,
    case
        when tarif.alternative_operator_name is not null then tarif.alternative_operator_name
        else msp.legacy_id
    end as "legacy_id!",
    charge_price.price,
    tarif.monthly_fee as monthly_fee,
    charge_price.updated
from charge_price join cpo on cpo.id = charge_price.cpo_id
                  join tarif on tarif.id = charge_price.tarif_id
                  join msp on tarif.msp_id = msp.id
where
    cpo.name = $1 and
    cpo.is_enabled and
    charge_price.c_type::text ilike $2
order by price, msp.name desc, tarif.slug_name