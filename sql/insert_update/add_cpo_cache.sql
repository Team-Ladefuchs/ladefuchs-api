INSERT INTO cpo_cache(network, slug_name, url, updated) Values($1, $2, $3, $4) ON CONFLICT (network) DO NOTHING
