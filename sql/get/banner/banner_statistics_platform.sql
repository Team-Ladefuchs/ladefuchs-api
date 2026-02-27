SELECT
    COALESCE(
        SUM(CASE WHEN platform = 'Android' THEN count ELSE 0 END), 0
    ) AS "android!",
    COALESCE(SUM(CASE WHEN platform = 'IOS' THEN count ELSE 0 END), 0) AS "ios!",
    COALESCE(SUM(CASE WHEN platform = 'Web' THEN count ELSE 0 END), 0) AS "web!"
FROM affiliate_statistic_daily
WHERE link_id = $1
