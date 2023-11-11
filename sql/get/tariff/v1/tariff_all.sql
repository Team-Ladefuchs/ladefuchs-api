SELECT 
    pub_tariff_id as identifier,
	slug_name as name,
    	jsonb_build_object(
		'identifier', tariff.provider_id, 
		'name', tariff.provider_name, 
		'customerOnly', tariff.provider_customer_only
	)::json as "provider!",
    monthly_fee,
    note,
    url,
    CASE 
        WHEN image.soft_delete = false THEN $1 || 'img/card/' || image.checksum
        ELSE null
    END as image,
    CASE 
        WHEN tariff.override_standard = true THEN tariff.override_standard
        ELSE tariff.standard
    END as "standard!"
FROM 
     tariff LEFT JOIN image ON tariff.image = image.id
WHERE hide = false
ORDER BY 
	slug_name, provider_name
