INSERT INTO tarif(msp_id, relationship_id, slug_name, monthly_fee, url)
VALUES($1, $2, $3 ,$4, $5) 
ON CONFLICT (relationship_id)
    DO UPDATE 
    SET 
        slug_name = excluded.slug_name,
        monthly_fee = excluded.monthly_fee,
        url = excluded.url
RETURNING id