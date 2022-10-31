with weeks as (select
    date_trunc('week', visited)::date as week,
    count(1) as clicks
from affiliate_statistic
where link_banner_id = $1 or link_banner_id is null
group by 1)


select avg(clicks)::bigint from weeks;
