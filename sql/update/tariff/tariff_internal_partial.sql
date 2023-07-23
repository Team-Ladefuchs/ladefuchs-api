update tariff
set 
	note = $2,
	is_enabled = $3
where id = $1
