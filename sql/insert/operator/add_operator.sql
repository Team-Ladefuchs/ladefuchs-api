INSERT INTO operator (network, name, slug_name, url, updated, standard, evse_id)
VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (network) DO NOTHING
