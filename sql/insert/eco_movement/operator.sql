INSERT INTO eco_movement.operator (id, name, website, ema_id)
VALUES ($1, $2, $3, $4)
ON CONFLICT DO NOTHING;
