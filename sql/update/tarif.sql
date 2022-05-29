UPDATE tarif
    SET 
        slug_name = $2,
        monthly_fee = $3,
        url = $4
WHERE id = $1