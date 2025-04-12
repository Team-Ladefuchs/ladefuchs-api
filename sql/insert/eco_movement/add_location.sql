insert into eco_movement.location (id, value) values (
    $1, $2
) on conflict do nothing;
