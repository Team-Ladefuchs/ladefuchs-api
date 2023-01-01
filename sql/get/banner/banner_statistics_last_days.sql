select
    count(id)
from affiliate_statistic
where visited > now() - $1::interval and link_id = $2 

