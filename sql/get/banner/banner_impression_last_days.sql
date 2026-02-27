select coalesce(sum(count), 0) as "total!"
from impression_banner_daily
where day >= current_date - $1::interval;
