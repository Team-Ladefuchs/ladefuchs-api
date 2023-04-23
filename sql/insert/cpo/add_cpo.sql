insert into cpo(name, slug_name, network, is_enabled, supported_types, power_ac, power_dc, hide) 
values ($1, $2, $3, $4, $5::chargeType[], $6, $7, true) 
returning id
