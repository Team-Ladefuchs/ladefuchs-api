select
    id,
    network,
    pub_network,
    slug_name,
    name,
    standard,
    updated,
    url,
    image,
    evse_id,
    not exists (
        select operator_id from charge_price
        where operator_id = operator.id
    ) as "hide!"
from operator
where
    search @@ websearch_to_tsquery($1)
    or strict_word_similarity($1, slug_name) > 0.55
