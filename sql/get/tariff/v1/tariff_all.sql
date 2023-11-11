SELECT 
    pub_tariff_id as identifier,
	slug_name as name,
	provider_name,
	provider_customer_only as is_customer_only,
    monthly_fee,
    note,
    url as affiliate_link_url,
    CASE 
        WHEN image.soft_delete = false THEN $1 || 'img/card/' || image.checksum
        ELSE null
    END as image_url,
    CASE 
        WHEN tariff.override_standard = true THEN tariff.override_standard
        ELSE tariff.standard
    END as "is_standard!",
	tariff.updated as last_updated_date
FROM 
     tariff LEFT JOIN image ON tariff.image = image.id
WHERE hide = false
ORDER BY 
	slug_name, provider_name
