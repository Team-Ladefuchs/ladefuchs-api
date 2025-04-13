insert into eco_movement.price (id, value) values (
    $1, $2
) on conflict do nothing;
