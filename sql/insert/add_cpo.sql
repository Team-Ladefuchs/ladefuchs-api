insert into cpo(name, slug_name, network, is_enabled, supported_types, power_ac, power_dc) 
values ($1, $2, $3, $4, $5, $6, $7) 
returning id
