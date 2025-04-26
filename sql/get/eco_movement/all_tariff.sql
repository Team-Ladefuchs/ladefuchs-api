SELECT
    id AS network,
    name,
    description,
    provider_name,
    type AS "tariff_type: TariffType",
    CASE
        WHEN subscription_fee_excl_vat::DOUBLE PRECISION > 0
            THEN (subscription_fee_excl_vat::DOUBLE PRECISION * 1.19)
        ELSE 0
    END AS subscription_fee
FROM eco_movement.tariff;
