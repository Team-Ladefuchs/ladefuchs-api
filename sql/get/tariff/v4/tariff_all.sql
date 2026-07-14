SELECT
    pub_tariff_id AS identifier,
    slug_name AS name,
    provider_name,
    provider_customer_only AS is_customer_only,
    monthly_fee,
    note,
    url AS affiliate_link_url,
    tariff.updated AS last_updated_date,
    ad_hoc AS is_ad_hoc,
    EXISTS (
        SELECT 1
        FROM dynamic_charge_price AS dp
        INNER JOIN location_dynamic_price AS ldp ON ldp.dynamic_price_id = dp.id
        WHERE dp.tariff_id = tariff.id
    ) AS "is_dynamic!",
    CASE
        WHEN image.soft_delete = false THEN $1 || 'image/' || image.checksum
    END AS image_url,
    coalesce(
        tariff.standard
        OR (
            tariff.brand_only = false
            AND (
                tariff.monthly_fee = 0.0
                AND tariff.provider_customer_only = false
            )
        ),
        false
    ) AS "is_standard!"
FROM
    tariff
LEFT JOIN image ON tariff.image = image.id
WHERE
    hide = false
    AND (
        EXISTS (
            SELECT 1 FROM charge_price
            WHERE charge_price.tariff_id = tariff.id
        )
        OR EXISTS (
            SELECT 1
            FROM dynamic_charge_price AS dp
            INNER JOIN location_dynamic_price AS ldp
                ON ldp.dynamic_price_id = dp.id
            WHERE dp.tariff_id = tariff.id
        )
    )
ORDER BY
    slug_name, provider_name
