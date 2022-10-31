insert into
  charge_price(
    cpo_id,
    tariff_id,
    c_type,
    price,
    blocking_fee_start
  )
values
  ($1, $2, $3, $4, $5) on conflict(cpo_id, tariff_id, c_type) do
update
set
  price = excluded.price,
  blocking_fee_start = excluded.blocking_fee_start,
  updated = now()
