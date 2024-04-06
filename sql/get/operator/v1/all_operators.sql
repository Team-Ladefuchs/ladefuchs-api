SELECT 
    CONCAT('cpo-', LOWER(name)) AS "identifier!",
    LOWER(name) AS "name!",
    slug_name AS display_name
FROM 
    operator 
WHERE 
    EXISTS (SELECT operator_id FROM charge_price WHERE operator_id = id) 
		AND  $1 != '' 
		AND ccs_plug_count > 0 or type2_plug_count > 0
ORDER BY 
    operator.name;
