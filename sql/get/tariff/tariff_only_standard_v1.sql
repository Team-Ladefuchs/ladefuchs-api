SELECT 
    pub_tariff_id as identifier,
    provider_name,
	slug_name as name,
    provider_id as provider_identifier,
    provider_customer_only,
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
    tariff 
JOIN 
    image 
ON 
    tariff.image = image.id 
WHERE 
    tariff.standard OR tariff.override_standard
ORDER BY 
	slug_name, provider_name
