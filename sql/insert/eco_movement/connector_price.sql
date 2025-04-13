INSERT INTO eco_movement.connector_price (
    location_id, pricing_id, evse_uid, connector_id
)
VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING;
