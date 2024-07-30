select
    count(id)as "total!"
from impression_banner
where visited::date >= CURRENT_DATE - $1::interval;
