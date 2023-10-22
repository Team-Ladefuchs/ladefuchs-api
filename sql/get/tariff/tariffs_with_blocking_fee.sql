select distinct operator.network as cpo_network,
                operator.name as cpo_name,
                operator.id as cpo_id,
                t.id as tariff_id,
                t.relationship_id
from charge_price cp
    join operator ON operator.id = cpo_id join tariff t on t.id = tariff_id
where blocking_fee_start > 0 and cp.is_protected = false and cpo_id = any($1);
