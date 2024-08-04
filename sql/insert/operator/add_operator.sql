INSERT INTO operator (network, name, slug_name, url, updated, standard)
VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (network) DO NOTHING
