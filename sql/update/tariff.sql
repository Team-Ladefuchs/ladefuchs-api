UPDATE tariff
	set 
        slug_name = $2,
        monthly_fee = $3,
        url = $4,
        updated = now()
WHERE id = $1
