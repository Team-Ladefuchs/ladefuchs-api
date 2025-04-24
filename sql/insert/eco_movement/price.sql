insert into eco_movement.price (id, provider_name, tariff_id, elements) values (
    $1, $2, $3, $4
) on conflict do nothing;
