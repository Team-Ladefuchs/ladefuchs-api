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
LEFT JOIN customer AS c ON link_banner.customer_id = c.id
WHERE
    ($1::text IS NULL OR link_banner.status::text = $1)
    AND starts <= now()
    AND expiration >= now()
    AND (
        c.total_impressions = 0  -- 0 = unlimited
        OR c.total_impressions > (
            SELECT COALESCE(COUNT(ib.id), 0)
            FROM impression_banner ib
            INNER JOIN link_banner lb2 ON ib.banner_link = lb2.id
            WHERE lb2.customer_id = link_banner.customer_id
        )
    )
