select
    charge_price.c_type as "charging_mode: ChargeType",
    tariff.pub_tariff_id as tariff_id,
    tariff.slug_name as tariff_name,
    charge_price.price as price_per_kwh,
    charge_price.blocking_fee,
    charge_price.blocking_fee_start,
    charge_price.updated
from charge_price inner join operator on charge_price.operator_id = operator.id
inner join tariff on charge_price.tariff_id = tariff.id
where
    operator.pub_network = $1
    and charge_price.c_type = any($2)
    and (
        tariff.standard
        or tariff.override_standard
        or tariff.pub_tariff_id = any($3)
    )
    and tariff.hide = false
order by price, tariff.slug_name;
