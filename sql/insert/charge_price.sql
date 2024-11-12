insert into
charge_price (
    operator_id,
    tariff_id,
    c_type,
    price,
    blocking_fee_start,
    blocking_fee
)
values
((select id from operator where network = $1), (select id from tariff where relationship_id = $2), $3, $4, $5, $6) on conflict (operator_id, tariff_id, c_type) do
update
set
price = excluded.price,
blocking_fee_start = excluded.blocking_fee_start,
blocking_fee = excluded.blocking_fee,
updated = now()
