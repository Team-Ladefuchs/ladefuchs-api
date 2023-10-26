update tariff
set 
	note = $2,
	standard = $3,
	internal_name = $4
where id = $1
