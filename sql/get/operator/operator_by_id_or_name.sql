select id, slug_name
from operator
where lower(operator.name) = lower($1) or operator.pub_network::text = $1
