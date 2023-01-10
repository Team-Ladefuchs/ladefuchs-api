select distinct c.network as cpo_network,
                c.name as cpo_name,
                c.id as cpo_id,
                t.id as tariff_id,
                t.relationship_id
from charge_price cp
    join cpo c ON c.id = cpo_id join tariff t on t.id = tariff_id
where blocking_fee_start > 0 and is_proteced = false;
