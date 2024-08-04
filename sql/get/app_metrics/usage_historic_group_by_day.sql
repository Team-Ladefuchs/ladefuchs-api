WITH platform_counts AS (
    SELECT
        visited::date AS visited,
        COUNT(DISTINCT CASE WHEN platform = 'IOS' THEN app_id END) AS ios,
        COUNT(
            DISTINCT CASE WHEN platform = 'Android' THEN app_id END
        ) AS android
    FROM
        app_metrics
    WHERE
        visited::date >= CURRENT_DATE - $1::interval
        AND visited::date < CURRENT_DATE
    GROUP BY
        visited::date
)

SELECT
    visited AS "visit_date!",
    ios AS "ios!",
    android AS "android!",
    ios + android AS "total!"
FROM
    platform_counts
ORDER BY
    visited;
