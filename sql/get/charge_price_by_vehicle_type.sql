select 
    msp.pub_msp_id as identifier,
    t.slug_name as name,
    msp.name as provider,
    charge_price.price,
    t.monthly_fee as monthly_fee,
    charge_price.updated
from charge_price join cpo on cpo.id = charge_price.cpo_id
                  join tarif t on t.id = charge_price.tarif_id
                  join vehicle_tarif vt on t.id = vt.tarif_id
                  join vehicle v on v.id = vt.vehicle_id
                  join msp on t.msp_id = msp.id
where
    cpo.name = $1 and
    cpo.is_enabled and
    v.vehicle_type::text = $2 and
    charge_price.c_type::text ilike $3 
order by price, msp.name desc, t.slug_name