insert into eco_movement.location (id, value, type, operator_id) values (
    $1, $2, $3, $4
) on conflict do nothing;
