SELECT
    COALESCE(
        SUM(CASE WHEN platform = 'Android' THEN 1 ELSE 0 END), 0
    ) AS "android!",
    COALESCE(SUM(CASE WHEN platform = 'IOS' THEN 1 ELSE 0 END), 0) AS "ios!",
    COALESCE(SUM(CASE WHEN platform = 'Web' THEN 1 ELSE 0 END), 0) AS "web!"
FROM affiliate_statistic
WHERE link_id = $1
