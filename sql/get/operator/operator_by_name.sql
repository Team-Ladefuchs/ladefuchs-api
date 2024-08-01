select id
from operator
where lower(operator.name) = lower($1)
