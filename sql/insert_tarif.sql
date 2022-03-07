INSERT INTO tarif (msp_id, relationship_id, vehicle_id, slug_name, monhtly_fee)
VALUES($1, $2, $3 ,$4, $5)
ON CONFLICT (relationship_id)
    DO
        UPDATE SET 
        relationship_id = excluded.relationship_id,
        slug_name = excluded.slug_name
RETURNING id