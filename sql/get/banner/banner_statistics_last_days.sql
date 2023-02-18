select
    count(id) as "count!"
from affiliate_statistic
where visited > now() - $1::interval and link_id = $2 

