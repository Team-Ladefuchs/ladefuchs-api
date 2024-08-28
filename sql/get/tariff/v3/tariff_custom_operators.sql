SELECT DISTINCT ON (pub_tariff_id)
    pub_tariff_id AS identifier,
    tariff.slug_name AS name,
    provider_name,
    provider_customer_only AS is_customer_only,
    monthly_fee,
    note,
    tariff.url AS affiliate_link_url,
    tariff.updated AS last_updated_date,
    CASE
        WHEN image.soft_delete = false THEN $1 || 'image/' || image.checksum
    END AS image_url,
    (
        o.pub_network = any($4)
        AND tariff.monthly_fee = 0
        AND tariff.provider_customer_only = false

    ) OR tariff.standard AS "is_standard!"
FROM
    tariff
LEFT JOIN image ON tariff.image = image.id
INNER JOIN public.charge_price AS cp ON tariff.id = cp.tariff_id
INNER JOIN public.operator AS o ON cp.operator_id = o.id
WHERE
    (
        tariff.standard
        OR
        (
            o.pub_network = any($4)
            AND tariff.monthly_fee = 0
            AND tariff.provider_customer_only = false
        )
        OR tariff.pub_tariff_id = any($2)
    )
    AND (tariff.pub_tariff_id != all($3))
    AND tariff.hide = false

ORDER BY
    pub_tariff_id, tariff.slug_name, provider_name
