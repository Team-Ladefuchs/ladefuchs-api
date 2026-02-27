select coalesce(sum(count), 0) as "count!"
from affiliate_statistic_daily
where day > (now() - $1::interval)::date and link_id = $2
