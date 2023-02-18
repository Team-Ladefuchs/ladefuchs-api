SELECT
       COALESCE(sum(case when platform = 'Android' then 1 else 0 end), 0) as "android!",
       COALESCE(sum(case when  platform = 'IOS' then 1 else 0 end), 0) as "ios!",
       COALESCE(sum(case when  platform = 'Web' then 1 else 0 end), 0) as "web!"
from affiliate_statistic
where link_id = $1
