select m.msp_id as identifier, t.slug_name as name,m.name as provider, charge_price.price, t.monhtly_fee as monthly_fee, updated::text
from charge_price join cpo c on c.id = charge_price.cpo_id
    join tarif t on t.id = charge_price.tarif_id 
    join msp m on t.msp_id = m.id
where charge_price.c_type::text = $1 and c.name= $2
order by m.name, c.slug_name
