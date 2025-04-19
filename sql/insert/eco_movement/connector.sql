INSERT INTO eco_movement.connector (
    id, evse_uid, power_type, max_power, connector_type
) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING;
