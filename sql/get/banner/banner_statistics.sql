with days as (
    select day::date as day
    from
        generate_series(now() - $1::interval, now(), interval '1 day') as t (
            day
        )
)

select
    days.day::timestamptz as "day!",
    coalesce(sum(asd.count), 0) as "clicks!"
from days
left join affiliate_statistic_daily asd on asd.day = days.day and asd.link_id = $2
group by days.day
order by days.day;
