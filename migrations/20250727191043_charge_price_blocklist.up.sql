-- Add up migration script here
create table if not exists charge_price_blocklist
(
    operator_id int NOT NULL REFERENCES operator (id) ON DELETE CASCADE,
    tariff_id   int NOT NULL REFERENCES tariff (id) ON DELETE CASCADE,
    PRIMARY KEY (operator_id, tariff_id)
);
