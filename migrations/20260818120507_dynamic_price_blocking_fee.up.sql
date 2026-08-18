ALTER TABLE dynamic_charge_price DROP CONSTRAINT dynamic_charge_price_key;

ALTER TABLE dynamic_charge_price
    ADD CONSTRAINT dynamic_charge_price_key
    UNIQUE NULLS NOT DISTINCT (operator_id, tariff_id, c_type, price,
                               blocking_fee_start, blocking_fee,
                               day_of_week, start_time, end_time, valid_from, valid_until);
