SELECT id, pub_network FROM cpo WHERE pub_network = any($1) and hide = false and cpo.is_enabled
