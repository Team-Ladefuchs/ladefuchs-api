INSERT INTO eco_movement.connector (
    id, evse_uid, power_type, max_power
) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING;
