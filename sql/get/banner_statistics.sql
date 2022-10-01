with days as (select day::date as day
              from generate_series(now() - $1::interval, now(), interval  '1 day') AS t(day))
SELECT
        days.day::timestamptz as "day!",
        count(id) as "clicks!"
FROM affiliate_state right join days on visited::date = days.day
WHERE
 link_banner_id = $2 or link_banner_id is null
GROUP BY days.day 
ORDER BY days.day;


