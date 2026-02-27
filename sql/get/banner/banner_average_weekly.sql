with weeks as (
    select
        date_trunc('week', day)::date as week,
        sum(count) as clicks
    from affiliate_statistic_daily
    where link_id = $1
    group by 1
)

select coalesce(avg(clicks)::bigint, 0) as "clicks!" from weeks;
