with days as (
    select day::date as day
    from
        generate_series(now() - $1::interval, now(), interval '1 day') as t (
            day
        )
)

select
    days.day::timestamptz as "day!",
    count(id) as "clicks!"
from affiliate_statistic right join days on visited::date = days.day
where link_id = $2
group by days.day
order by days.day;
