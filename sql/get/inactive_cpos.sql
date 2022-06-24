select id, name::text
from cpo
where not exists (select cpo_id from charge_price where cpo_id = id) and is_enabled = true