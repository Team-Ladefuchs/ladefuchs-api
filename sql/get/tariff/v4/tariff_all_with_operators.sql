SELECT
    tariff.pub_tariff_id AS identifier,
    tariff.slug_name AS name,
    tariff.provider_name,
    tariff.provider_customer_only AS is_customer_only,
    tariff.monthly_fee,
    tariff.note,
    tariff.url AS affiliate_link_url,
    tariff.updated AS last_updated_date,
    tariff.ad_hoc AS is_ad_hoc,
    EXISTS (
        SELECT 1
        FROM dynamic_charge_price AS dp
        INNER JOIN location_dynamic_price AS ldp ON ldp.dynamic_price_id = dp.id
        WHERE dp.tariff_id = tariff.id
    ) AS "is_dynamic!",
    CASE
        WHEN image.soft_delete = FALSE THEN $1 || 'image/' || image.checksum
    END AS image_url,
    (
        (
            EXISTS (
                SELECT 1
                FROM charge_price AS cp
                INNER JOIN operator AS o ON cp.operator_id = o.id
                WHERE cp.tariff_id = tariff.id
                    AND o.pub_network = ANY($2)
            )
            OR EXISTS (
                SELECT 1
                FROM dynamic_charge_price AS dp
                INNER JOIN operator AS o ON dp.operator_id = o.id
                INNER JOIN location_dynamic_price AS ldp
                    ON ldp.dynamic_price_id = dp.id
                WHERE dp.tariff_id = tariff.id
                    AND o.pub_network = ANY($2)
            )
        )
        AND tariff.monthly_fee = 0
        AND tariff.provider_customer_only = FALSE
    ) OR tariff.standard AS "is_standard!"
FROM
    tariff
LEFT JOIN image ON tariff.image = image.id
WHERE
    tariff.hide = FALSE
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
    tariff.slug_name, tariff.provider_name;
