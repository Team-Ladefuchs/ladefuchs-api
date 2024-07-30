WITH distinct_counts AS (
    SELECT
        COUNT(DISTINCT CASE WHEN platform = 'IOS' THEN app_id END) as ios,
        COUNT(DISTINCT CASE WHEN platform = 'Android' THEN app_id END) as android
    FROM
        app_metrics
    WHERE
        visited::date >= CURRENT_DATE - $1::interval
)
SELECT
    ios as "ios!",
    android as "android!",
    ios + android AS "total!"
FROM
    distinct_counts;
