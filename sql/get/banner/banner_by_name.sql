select id
from link_banner
where lower(link_banner.name) = lower($1)
