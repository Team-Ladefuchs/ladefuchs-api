SELECT
       sum(case when plattform = 'Android' then 1 else 0 end) as "android!",
       sum(case when  plattform = 'IOS' then 1 else 0 end) as "ios!",
       sum(case when  plattform = 'Web' then 1 else 0 end) as "web!"
from affiliate_state
where link_banner_id = $1

