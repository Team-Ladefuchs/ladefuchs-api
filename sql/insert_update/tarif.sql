INSERT INTO tarif(msp_id, relationship_id, slug_name, monthly_fee)
VALUES($1, $2, $3 ,$4)
ON CONFLICT (relationship_id)
    DO UPDATE SET slug_name = excluded.slug_name
RETURNING id