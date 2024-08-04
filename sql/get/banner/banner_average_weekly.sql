with weeks as (
    select
        date_trunc('week', visited)::date as week,
        count(1) as clicks
    from affiliate_statistic
    where link_id = $1
    group by 1
)


select coalesce(avg(clicks)::bigint, 0) as "clicks!" from weeks;
