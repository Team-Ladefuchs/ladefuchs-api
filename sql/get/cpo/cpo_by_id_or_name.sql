select id
from cpo
where (lower(cpo.name) = lower($1) or cpo.pub_network::text = $1)
