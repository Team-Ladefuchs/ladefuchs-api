INSERT INTO eco_movement.provider (id, name)
VALUES ($1, $2)
ON CONFLICT DO NOTHING
