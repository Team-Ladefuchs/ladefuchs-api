update tariff
set 
	note = $2,
	is_enabled = $3,
	internal_name = $4
where id = $1
