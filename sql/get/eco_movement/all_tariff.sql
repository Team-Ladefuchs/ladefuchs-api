select
    id as network,
    name,
    description,
    provider_name,
    type as "tariff_type: TariffType",
    subscription_fee_excl_vat::DOUBLE PRECISION
from eco_movement.tariff
