
WITH platform_counts AS (
    SELECT
        visited::date as visited,
        COUNT(DISTINCT CASE WHEN platform = 'IOS' THEN app_id END) AS ios,
        COUNT(DISTINCT CASE WHEN platform = 'Android' THEN app_id END) AS android
    FROM
        app_metrics
    WHERE
        visited::date >= CURRENT_DATE - $1::interval
    GROUP BY
        visited::date
)
SELECT
    visited as "visit_date!",
    ios as "ios!",
    android as "android!",
    ios + android AS "total!"
FROM
    platform_counts
ORDER BY
    visited;
