WITH distinct_counts AS (
    SELECT
        COUNT(DISTINCT CASE WHEN platform = 'IOS' THEN app_id END) AS ios,
        COUNT(
            DISTINCT CASE WHEN platform = 'Android' THEN app_id END
        ) AS android
    FROM
        app_metrics
    WHERE
        visited::date >= CURRENT_DATE - $1::interval
)

SELECT
    ios AS "ios!",
    android AS "android!",
    ios + android AS "total!"
FROM
    distinct_counts;
