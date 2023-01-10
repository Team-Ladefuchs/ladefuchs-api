update charge_price
set blockingFee = $1
where cpo_id = $2 and tariff_id = $3 and c_type = $4
