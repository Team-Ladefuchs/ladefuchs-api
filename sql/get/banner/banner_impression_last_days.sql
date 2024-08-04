select count(id) as "total!"
from impression_banner
where visited::date >= current_date - $1::interval;
