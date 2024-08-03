select
    charge_price.c_type as "charging_mode: ChargeType",
    tariff.pub_tariff_id as tariff_id,
    tariff.slug_name as tariff_name,
    charge_price.price as price_per_kwh,
    charge_price.blocking_fee,
    charge_price.blocking_fee_start,
	charge_price.updated
from charge_price join operator on operator.id = charge_price.operator_id
                  join tariff on tariff.id = charge_price.tariff_id
where
        operator.pub_network = $1 AND
		charge_price.c_type = any($2) AND
		tariff.pub_tariff_id = any($3) AND
		tariff.hide = false
order by price, tariff.slug_name;
