SELECT
       sum(case when platform = 'Android' then 1 else 0 end) as "android!",
       sum(case when  platform = 'IOS' then 1 else 0 end) as "ios!",
       sum(case when  platform = 'Web' then 1 else 0 end) as "web!"
from affiliate_statistic
where link_banner_id = $1

