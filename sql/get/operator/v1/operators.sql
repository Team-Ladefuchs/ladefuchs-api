SELECT 
    CONCAT('cpo-', LOWER(name)) AS "identifier!",
    LOWER(name) AS "name!",
    slug_name AS display_name
FROM 
    operator 
WHERE 
    EXISTS (SELECT cpo_id FROM charge_price WHERE cpo_id = id) AND 
    (is_enabled = $1 AND $2 != '')
ORDER BY 
    operator.name;
