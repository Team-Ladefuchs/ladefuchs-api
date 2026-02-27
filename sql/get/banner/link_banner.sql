WITH customer_impressions AS (
    SELECT lb.customer_id, COALESCE(SUM(ibd.count), 0) AS total
    FROM impression_banner_daily ibd
    INNER JOIN link_banner lb ON ibd.banner_link = lb.id
    GROUP BY lb.customer_id
)
SELECT
    link_banner.pub_id AS id,
    l.source,
    l.is_affiliate,
    link_banner.frequency,
    i.checksum,
    replace(i.file_path, ' ', '') AS "image!",
    greatest(link_banner.updated, i.updated) AS "updated!"
FROM link_banner
INNER JOIN link AS l ON link_banner.link_id = l.id
INNER JOIN image AS i ON link_banner.image = i.id
INNER JOIN customer AS c ON link_banner.customer_id = c.id
LEFT JOIN customer_impressions ci ON ci.customer_id = link_banner.customer_id
WHERE
    ($1::text IS NULL OR link_banner.status::text = $1)
    AND starts <= now()
    AND expiration >= now()
    AND (
        c.total_impressions = 0  -- 0 = unlimited
        OR c.total_impressions > COALESCE(ci.total, 0)
    )
