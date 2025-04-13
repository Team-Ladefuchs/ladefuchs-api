insert into eco_movement.tariff (id, value) values (
    $1, $2
) on conflict do nothing;
