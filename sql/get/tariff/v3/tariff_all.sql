SELECT
    pub_tariff_id AS identifier,
    slug_name AS name,
    provider_name,
    provider_customer_only AS is_customer_only,
    monthly_fee,
    note,
    url AS affiliate_link_url,
    tariff.updated AS last_updated_date,
    CASE
        WHEN image.soft_delete = false THEN $1 || 'image/' || image.checksum
    END AS image_url,
    CASE
        WHEN tariff.override_standard = true THEN tariff.override_standard
        ELSE tariff.standard
    END AS "is_standard!"
FROM
    tariff
LEFT JOIN image ON tariff.image = image.id
WHERE
    hide = false
    AND EXISTS (SELECT tariff_id FROM charge_price WHERE tariff_id = tariff.id)
ORDER BY
    slug_name, provider_name
