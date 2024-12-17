WITH query AS (
    SELECT DISTINCT ON (tariff.pub_tariff_id)
        tariff.pub_tariff_id AS identifier,
        tariff.slug_name AS name,
        tariff.provider_name,
        tariff.provider_customer_only AS is_customer_only,
        tariff.monthly_fee,
        tariff.note,
        tariff.url AS affiliate_link_url,
        tariff.updated AS last_updated_date,
        CASE
            WHEN image.soft_delete = FALSE THEN $1 || 'image/' || image.checksum
        END AS image_url,
        (
            -- Condition for tariff to be considered standard or applicable
            (
                o.pub_network = ANY($4)
                AND tariff.monthly_fee = 0
                AND tariff.provider_customer_only = FALSE
                AND tariff.slug_name NOT ILIKE '%business%'
            )
            OR tariff.standard
            OR tariff.brand_only
        ) AS "is_standard!"
    FROM
        tariff
    LEFT JOIN image ON tariff.image = image.id
    INNER JOIN public.charge_price AS cp ON tariff.id = cp.tariff_id
    INNER JOIN public.operator AS o ON cp.operator_id = o.id
    WHERE
        -- Condition to check whether tariff is standard, matches operator network, or is specific tariff by pub_tariff_id
        (
            (
                tariff.standard
                OR (
                    tariff.monthly_fee = 0
                    AND tariff.provider_customer_only = FALSE
                    AND tariff.brand_only = FALSE
                    AND tariff.slug_name NOT ILIKE '%business%'
                )
                AND o.pub_network = ANY($4)
            )
            OR tariff.pub_tariff_id = ANY($2)
        )
        AND tariff.pub_tariff_id != ALL($3)
        AND tariff.hide = FALSE
    ORDER BY
        tariff.pub_tariff_id
)

SELECT *
FROM query
ORDER BY
    name, provider_name;
