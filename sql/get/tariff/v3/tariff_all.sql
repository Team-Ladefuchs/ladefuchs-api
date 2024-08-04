SELECT
    pub_tariff_id AS identifier,
    slug_name AS name,
    provider_name,
    provider_customer_only AS is_customer_only,
    monthly_fee,
    note,
    url AS affiliate_link_url,
    CASE
        WHEN image.soft_delete = false THEN $1 || 'image/' || image.checksum
        ELSE null
    END AS image_url,
    CASE
        WHEN tariff.override_standard = true THEN tariff.override_standard
        ELSE tariff.standard
    END AS "is_standard!",
    tariff.updated AS last_updated_date
FROM
    tariff
LEFT JOIN image ON tariff.image = image.id
WHERE hide = false
ORDER BY
    slug_name, provider_name
