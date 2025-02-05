select
    operator.slug_name,
    operator.id
from operator
where operator.pub_network = $1
