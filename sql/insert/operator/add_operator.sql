INSERT INTO operator(network, name, slug_name, url, updated, is_enabled) 
Values($1, $2, $3, $4, $5, $6) ON CONFLICT (network) DO NOTHING
